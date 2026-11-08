-- Nachzug: stellt `booking.created_by` wieder her, wo ein Slot-Umbau es durch
-- 'system' ersetzt hat.
--
-- HINTERGRUND
--
-- Die `*`-Markierung im Schichtplan zeigt dem Schichtplaner, wer sich selbst
-- eingetragen hat. Sie haengt an `ShiftplanBooking::self_added`, und das wird
-- nicht gespeichert, sondern in `service_impl/src/shiftplan.rs` abgeleitet aus
-- `booking.created_by == <Benutzer dieser Sales Person>`.
--
-- Bis zum Fix in `shiftplan_edit.rs` haben beide Umbau-Pfade (`modify_slot`,
-- `modify_slot_single_week`) die Buchungen eines Slots geloescht und neu
-- angelegt und dabei `created_by` zurueckgesetzt. Die Fallback-Kette in
-- `BookingService::create` stempelte dann 'system', weil `current_user` bei
-- `Authentication::Full` None ist. Die Urheberschaft ist dadurch nicht verloren
-- — sie steht weiterhin in der soft-geloeschten Vorgaenger-Zeile.
--
-- VERFAHREN
--
-- Es gibt keinen Fremdschluessel von der neuen auf die alte Zeile: die neue
-- Buchung bekommt eine frische `id`, eine frische `update_version` und zeigt auf
-- den neuen Slot. Rekonstruiert wird ueber die Zeit:
--
--   1. Gruppieren nach (sales_person_id, year, calendar_week, slot.day_of_week).
--      Der `day_of_week` bleibt beim Umbau erhalten, weil `modify_slot` den
--      neuen Slot als Kopie des alten anlegt — damit bleiben mehrere Buchungen
--      derselben Person in derselben Woche unterscheidbar.
--   2. Innerhalb der Gruppe ist der Vorgaenger die Zeile mit dem groessten
--      `created` STRIKT KLEINER als das eigene. Der Zeitstempel dient zum
--      ORDNEN, nicht zum Verbinden: `delete()` und `create()` fragen die Uhr
--      getrennt ab, und `Iso8601::DATE_TIME` schreibt Nanosekunden — ein
--      Gleichheits-Join auf `Vorgaenger.deleted = Nachfolger.created` haette
--      null Treffer. Das Textformat ist festbreitig ISO8601, lexikografische
--      Ordnung entspricht damit der zeitlichen.
--   3. Rueckwaerts laufen, NUR ueber Glieder mit `deleted_by = 'system'`. Steht
--      dort ein echter Benutzername, hat ein Mensch geloescht — dann sind es
--      zwei getrennte Episoden und die Kette endet.
--
-- WAS AUSDRUECKLICH NICHT ANGEFASST WIRD
--
-- Nicht jedes 'system' ist Schaden. Der Auto-Rebooking-Cron
-- (`voluntary_rebooking_scheduler`) und andere Systempfade buchen legitim als
-- 'system'. Solche Zeilen haben keine system-geloeschte Vorgaengerkette, die zu
-- einem echten Namen fuehrt, und bleiben unveraendert. Ebenso unberuehrt:
--
--   * Ketten, die an einer Zeile ohne `created_by` enden (Altbestand vor
--     `20250115000000_add-user-tracking-to-booking.sql`),
--   * Ketten, die an einer Zeile ohne `deleted_by` enden (ebenfalls Altbestand —
--     ob Umbau oder Mensch ist dort nicht entscheidbar),
--   * Ketten, die durch eine Menschen-Loeschung getrennt sind,
--   * mehrdeutige Faelle: tragen zwei Vorgaenger dasselbe `created` und
--     VERSCHIEDENE Urheber, wird nichts geschrieben.
--
-- NACHTRAEGLICHER BERICHT
--
-- Der Nachzug hinterlaesst `update_process = 'backfill-booking-authorship'`.
-- Wie viele Zeilen er angefasst hat:
--
--   SELECT COUNT(*) FROM booking
--    WHERE update_process = 'backfill-booking-authorship';
--
-- Was er bewusst NICHT angefasst hat (lebende 'system'-Buchungen, die keiner
-- Kette zuzuordnen waren):
--
--   SELECT COUNT(*) FROM booking
--    WHERE deleted IS NULL AND created_by = 'system'
--      AND update_process <> 'backfill-booking-authorship';
--
-- IDEMPOTENZ
--
-- Die Bedingung `created_by = 'system' AND deleted IS NULL` macht einen zweiten
-- Durchlauf zum No-op. sqlx fuehrt eine Migration ohnehin nur einmal aus; die
-- Bedingung ist die zweite Verteidigungslinie und erlaubt es, den
-- BACKFILL-Abschnitt im Test gezielt ein zweites Mal zu fahren.

-- BACKFILL START
WITH RECURSIVE
    -- Alle Buchungen mit Gruppenschluessel. Soft-geloeschte Zeilen sind hier
    -- GEWOLLT — sie tragen die gesuchte Urheberschaft. Auch soft-geloeschte
    -- Slots bleiben drin: ein Umbau verkuerzt den alten Slot in der Regel nur,
    -- loescht ihn aber im Randfall (erste KW) tatsaechlich.
    chain AS (
        SELECT
            b.id             AS id,
            b.sales_person_id AS sales_person_id,
            b.year            AS year,
            b.calendar_week   AS calendar_week,
            s.day_of_week     AS day_of_week,
            b.created         AS created,
            b.deleted         AS deleted,
            b.created_by      AS created_by,
            b.deleted_by      AS deleted_by
        FROM booking b
        JOIN slot s ON s.id = b.slot_id
    ),

    -- Rueckwaerts-Lauf. `target_id` ist die lebende Buchung, fuer die wir einen
    -- Urheber suchen; `cursor_created` wandert mit jedem Schritt nach vorn in
    -- die Vergangenheit. `resolved_by` ist gesetzt, sobald ein echter Name
    -- gefunden ist — dann endet der Lauf fuer diese Zeile.
    walk(target_id, grp_sp, grp_year, grp_week, grp_dow, cursor_created, resolved_by) AS (
        SELECT
            c.id, c.sales_person_id, c.year, c.calendar_week, c.day_of_week,
            c.created, NULL
        FROM chain c
        WHERE c.deleted IS NULL
          AND c.created_by = 'system'

        UNION ALL

        SELECT
            w.target_id, w.grp_sp, w.grp_year, w.grp_week, w.grp_dow,
            p.created,
            CASE
                WHEN p.created_by IS NOT NULL
                 AND p.created_by <> 'system'
                 AND p.created_by <> ''
                THEN p.created_by
                ELSE NULL
            END
        FROM walk w
        JOIN chain p
          ON  p.sales_person_id = w.grp_sp
          AND p.year            = w.grp_year
          AND p.calendar_week   = w.grp_week
          AND p.day_of_week     = w.grp_dow
          AND p.created = (
                SELECT MAX(q.created)
                FROM chain q
                WHERE q.sales_person_id = w.grp_sp
                  AND q.year            = w.grp_year
                  AND q.calendar_week   = w.grp_week
                  AND q.day_of_week     = w.grp_dow
                  AND q.created         < w.cursor_created
              )
        -- Nur ueber Umbau-Glieder zurueck. `deleted_by` NULL bricht hier
        -- ebenfalls ab (Altbestand, nicht entscheidbar) — ein NULL-Vergleich
        -- ist in SQL nicht wahr, der Join liefert die Zeile also nicht.
        WHERE p.deleted_by = 'system'
          -- Nicht weiterlaufen, wenn bereits aufgeloest.
          AND w.resolved_by IS NULL
    ),

    -- Genau ein eindeutiger Urheber pro Zielzeile. Tragen zwei Vorgaenger auf
    -- derselben Stufe verschiedene Namen, faellt die Zeile hier heraus und
    -- bleibt unveraendert — geraten wird nicht.
    resolved AS (
        SELECT target_id, MIN(resolved_by) AS new_created_by
        FROM walk
        WHERE resolved_by IS NOT NULL
        GROUP BY target_id
        HAVING COUNT(DISTINCT resolved_by) = 1
    )

UPDATE booking
   SET created_by     = (SELECT new_created_by FROM resolved WHERE target_id = booking.id),
       update_version = randomblob(16),
       update_process = 'backfill-booking-authorship'
 WHERE deleted IS NULL
   AND created_by = 'system'
   AND id IN (SELECT target_id FROM resolved);
-- BACKFILL END
