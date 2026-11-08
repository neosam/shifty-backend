-- Nachzug v2: stellt `booking.created_by` wieder her, wo ein Slot-Umbau es
-- durch 'system' ersetzt hat -- und korrigiert, was v1 falsch zugeordnet hat.
--
-- WARUM EINE ZWEITE MIGRATION
--
-- `20261108000000_backfill-booking-authorship.sql` ist bereits ausgeliefert und
-- mit Pruefsumme in `_sqlx_migrations` eingetragen. Eine Aenderung an dieser
-- Datei laesst `sqlx::migrate!` beim Start mit einem Checksum-Fehler
-- abbrechen. Die Korrektur kommt deshalb als eigene Migration daneben.
--
-- WAS AN v1 FALSCH WAR
--
-- v1 gruppierte nach (sales_person_id, year, calendar_week, slot.day_of_week)
-- und nahm als Vorgaenger die Zeile mit dem groessten `created` darunter. Dieser
-- Schluessel wirft ALLE Slots eines Wochentags in einen Topf. Beobachtet auf der
-- Integrationsumgebung, eine Person an einem Freitag:
--
--   created                        slot        created_by  deleted_by
--   2026-10-02T12:36:16.201727600  16:00-17:00 jakob.l     -
--   2026-10-02T12:36:16.902174899  17:00-18:00 jakob.l     system
--   2026-10-02T12:36:17.652195120  18:00-19:00 jakob.l     system
--   2026-10-02T12:36:18.245657244  19:00-19:30 jakob.l     system
--   2026-11-08T07:17:18.220225594  17:00-18:00 system      -     <- Ziel
--   2026-11-08T07:17:24.104249595  18:00-19:00 system      -     <- Ziel
--   2026-11-08T07:17:27.757124631  19:00-19:30 system      -     <- Ziel
--
-- Fuer die 18:00-Zeile war der "Vorgaenger" nach v1 die 17:00-Zeile von
-- 07:17:18 -- die LEBT, ihr `deleted_by` ist NULL, und die Kette brach sofort
-- ab. Zwei der drei Zeilen blieben deshalb auf 'system' stehen. Die dritte
-- (17:00) wurde nur zufaellig richtig: sie holte ihren Urheber vom 19:00-Slot,
-- der demselben Menschen gehoerte. In einer gemischten Woche -- die planende
-- Person traegt einige Schichten ein, die Person sich selbst andere -- schreibt
-- v1 dort den FALSCHEN Namen.
--
-- DAS VERFAHREN IN v2
--
-- Verbunden wird nicht mehr ueber den Gruppenschluessel, sondern ueber die
-- Nachbarschaft der Zeitstempel. `modify_slot` laeuft pro Buchung
-- `delete()` und direkt danach `create()`; beide Aufrufe fragen die Uhr
-- getrennt ab, liegen aber nur Millisekunden auseinander. Aus denselben Daten:
--
--   deleted 2026-11-08T07:17:18.212075459 -> created 07:17:18.220225594   8,2 ms
--   deleted 2026-11-08T07:17:24.097474460 -> created 07:17:24.104249595   6,8 ms
--   deleted 2026-11-08T07:17:27.752902705 -> created 07:17:27.757124631   4,2 ms
--
-- Der Vorgaenger ist also die soft-geloeschte Zeile mit dem groessten `deleted`
-- NICHT NACH dem eigenen `created`, und zwar nur innerhalb eines Fensters von
-- einer Sekunde (beobachtet sind < 10 ms, das Fenster ist um Faktor 100
-- grosszuegig). Ein Gleichheits-Join waere falsch -- die Stempel sind nicht
-- gleich, nur fast gleich. `Iso8601::DATE_TIME` schreibt Nanosekunden und keinen
-- Zeitzonen-Zusatz; SQLites `julianday()` verarbeitet neun Nachkommastellen und
-- auch den Uebertritt der Sekundengrenze korrekt.
--
-- Die AUSWAHL des naechsten Vorgaengers laeuft trotzdem ueber den Text, nicht
-- ueber `julianday()`: ein julianischer Tag ist ein double und hat bei diesen
-- Groessenordnungen nur rund 9 Mikrosekunden Aufloesung -- zwei auf die
-- Nanosekunde verschiedene Stempel koennen darin zusammenfallen. Das Textformat
-- ist festbreitig, die lexikografische Ordnung entspricht damit der zeitlichen
-- und ist exakt. `julianday()` dient ausschliesslich dem Groessenvergleich gegen
-- das Ein-Sekunden-Fenster, wo Mikrosekunden keine Rolle spielen.
--
-- (sales_person_id, year, calendar_week, slot.day_of_week) bleibt als zusaetzliche
-- Absicherung erhalten -- `day_of_week` aendert sich beim Umbau nie, weil der neue
-- Slot als Kopie des alten entsteht. Die Slot-ZEIT taugt dagegen nicht als
-- Schluessel: `modify_slot` uebernimmt `from`/`to` aus dem Aufruf, ein Umbau darf
-- sie also veraendern.
--
-- Rueckwaerts gelaufen wird weiter NUR ueber Glieder mit `deleted_by = 'system'`.
--
-- WAS v2 ANFASST
--
--   * lebende Buchungen mit `created_by = 'system'` (von v1 nicht erreicht),
--   * lebende Buchungen, die v1 gestempelt hat -- sie werden neu berechnet und
--     nur geschrieben, wenn das Ergebnis vom aktuellen Wert ABWEICHT. v1 hat die
--     soft-geloeschten Vorgaenger nicht veraendert, die Urheberschaft steht dort
--     also unberuehrt zur Verfuegung.
--
-- Buchungen mit einem echten Urheber, die v1 NICHT gestempelt hat, sind tabu.
--
-- ZURUECKNAHME
--
-- Hat v1 eine Zeile gestempelt, die v2 nicht aufloesen kann, faellt sie auf
-- 'system' zurueck. Das ist Absicht und folgt derselben Abwaegung, auf der schon
-- v1 gebaut war: eine falsche Urheberschaft im Audit-Trail ist schlimmer als
-- eine fehlende `*`-Markierung. v1 hat aus einem groberen Schluessel geraten;
-- was v2 nicht bestaetigt, bleibt nicht stehen. Der sichtbare Preis ist eine
-- fehlende `*` in den Faellen, in denen v1 zufaellig richtig lag.
--
-- WAS AUSDRUECKLICH NICHT ANGEFASST WIRD
--
-- Unveraendert gegenueber v1: nicht jedes 'system' ist Schaden. Der
-- Auto-Rebooking-Cron (`voluntary_rebooking_scheduler`) bucht legitim als
-- 'system' und hat keinen system-geloeschten Vorgaenger im Zeitfenster. Ebenso
-- unberuehrt: Ketten, die an einer Zeile ohne `created_by` oder ohne
-- `deleted_by` enden (Altbestand), Ketten, die eine Menschen-Loeschung trennt,
-- und mehrdeutige Faelle (zwei Vorgaenger, verschiedene Urheber -> es wird
-- nicht geraten).
--
-- BERICHT
--
-- v2 stempelt denselben `update_process` wie v1, damit die bestehenden
-- Abfragen weiter funktionieren. `created_by` trennt die beiden Ausgaenge:
--
--   -- Urheberschaft wiederhergestellt
--   SELECT COUNT(*) FROM booking
--    WHERE update_process = 'backfill-booking-authorship'
--      AND created_by <> 'system';
--
--   -- von v2 zurueckgenommen (v1 hatte geraten, v2 bestaetigt es nicht)
--   SELECT COUNT(*) FROM booking
--    WHERE update_process = 'backfill-booking-authorship'
--      AND created_by = 'system';
--
-- IDEMPOTENZ
--
-- Das abschliessende `created_by IS NOT (...)` macht einen zweiten Durchlauf zum
-- echten No-op -- auch `update_version` bleibt dann unberuehrt.

-- BACKFILL START
WITH RECURSIVE
    -- Alle Buchungen mit Gruppenschluessel. Soft-geloeschte Zeilen sind hier
    -- GEWOLLT -- sie tragen die gesuchte Urheberschaft. Auch soft-geloeschte
    -- Slots bleiben drin: ein Umbau verkuerzt den alten Slot in der Regel nur,
    -- loescht ihn aber im Randfall (erste KW) tatsaechlich.
    chain AS (
        SELECT
            b.id              AS id,
            b.sales_person_id AS sales_person_id,
            b.year            AS year,
            b.calendar_week   AS calendar_week,
            s.day_of_week     AS day_of_week,
            b.created         AS created,
            b.deleted         AS deleted,
            b.created_by      AS created_by,
            b.deleted_by      AS deleted_by,
            b.update_process  AS update_process
        FROM booking b
        JOIN slot s ON s.id = b.slot_id
    ),

    -- Rueckwaerts-Lauf. `cursor_created` ist das `created` der jeweils erreichten
    -- Zeile und wandert mit jedem Schritt weiter in die Vergangenheit.
    -- `resolved_by` ist gesetzt, sobald ein echter Name gefunden ist -- dann
    -- endet der Lauf fuer diese Zielzeile.
    walk(target_id, grp_sp, grp_year, grp_week, grp_dow, cursor_created, resolved_by) AS (
        SELECT
            c.id, c.sales_person_id, c.year, c.calendar_week, c.day_of_week,
            c.created, NULL
        FROM chain c
        WHERE c.deleted IS NULL
          AND (
                c.created_by = 'system'
                -- von v1 gestempelt: neu berechnen und gegenpruefen
                OR c.update_process = 'backfill-booking-authorship'
              )

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
          -- Nur ueber Umbau-Glieder zurueck. `deleted_by` NULL bricht hier
          -- ebenfalls ab (Altbestand, nicht entscheidbar) -- ein NULL-Vergleich
          -- ist in SQL nicht wahr, der Join liefert die Zeile also nicht.
          AND p.deleted IS NOT NULL
          AND p.deleted_by      = 'system'
          -- Zeitliche Nachbarschaft: die Loeschung liegt unmittelbar vor dem
          -- eigenen Anlegen.
          AND p.deleted        <= w.cursor_created
          AND (julianday(w.cursor_created) - julianday(p.deleted)) * 86400.0 < 1.0
          -- Terminierung: der Cursor muss strikt zurueckwandern. Folgt aus
          -- created < deleted, ausser bei exakt gleichen Stempeln -- deshalb
          -- explizit.
          AND p.created         < w.cursor_created
          -- Der naechstgelegene Vorgaenger, nicht irgendeiner.
          AND p.deleted = (
                SELECT MAX(q.deleted)
                FROM chain q
                WHERE q.sales_person_id = w.grp_sp
                  AND q.year            = w.grp_year
                  AND q.calendar_week   = w.grp_week
                  AND q.day_of_week     = w.grp_dow
                  AND q.deleted IS NOT NULL
                  AND q.deleted_by      = 'system'
                  AND q.deleted        <= w.cursor_created
              )
        -- Nicht weiterlaufen, wenn bereits aufgeloest.
        WHERE w.resolved_by IS NULL
    ),

    -- Genau ein eindeutiger Urheber pro Zielzeile. Tragen zwei Vorgaenger auf
    -- derselben Stufe verschiedene Namen, faellt die Zeile hier heraus und
    -- bleibt unveraendert -- geraten wird nicht.
    resolved AS (
        SELECT target_id, MIN(resolved_by) AS new_created_by
        FROM walk
        WHERE resolved_by IS NOT NULL
        GROUP BY target_id
        HAVING COUNT(DISTINCT resolved_by) = 1
    )

UPDATE booking
   -- Loest v2 die Zeile auf, gilt ihr Ergebnis. Loest sie sie NICHT auf, faellt
   -- die Zeile auf 'system' zurueck -- siehe ZURUECKNAHME im Kopf.
   SET created_by     = COALESCE(
                            (SELECT new_created_by FROM resolved WHERE target_id = booking.id),
                            'system'
                        ),
       update_version = randomblob(16),
       update_process = 'backfill-booking-authorship'
 WHERE deleted IS NULL
   -- Zeilen mit echtem Urheber, die v1 nicht gestempelt hat, bleiben tabu.
   AND (created_by = 'system' OR update_process = 'backfill-booking-authorship')
   -- Nur schreiben, wenn sich etwas aendert. Macht den zweiten Lauf zum echten
   -- No-op (auch `update_version` bleibt stehen) und laesst von v1 korrekt
   -- gesetzte Zeilen unberuehrt.
   AND created_by IS NOT COALESCE(
                             (SELECT new_created_by FROM resolved WHERE target_id = booking.id),
                             'system'
                         );
-- BACKFILL END
