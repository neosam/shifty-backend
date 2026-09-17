-- Phase 61 (PERS-05) — Aufloesung der 1:1-Verknuepfungstabelle
-- `sales_person_user`. Ab hier traegt allein `person_id` auf `sales_person` und
-- `user` die Kopplung; alle lesenden und schreibenden Pfade wurden in den
-- Plaenen 61-03 und 61-04 darauf umgestellt.
--
-- D-61-06: Block A zieht den Zustand nach, der zwischen dem Phase-60- und dem
-- Phase-61-Deploy entstanden sein kann. Das sind ZWEI Klassen, nicht eine:
--   (1) Zeilen, die noch gar kein `person_id` tragen — D-60-10 laesst eine im
--       Fenster frisch angelegte `sales_person`-Zeile ohne Identitaet
--       ausdruecklich zu.
--   (2) Zeilen, die beide bereits eine EIGENE Person tragen, im Fenster aber
--       neu verknuepft oder umgehaengt wurden. Dort lief noch der alte
--       `assign_to_user`, der ausschliesslich eine `sales_person_user`-Zeile
--       schrieb und die beiden `person_id` unberuehrt liess.
-- Klasse (2) ist der REGELFALL des Fensters, nicht der Ausnahmefall: nach dem
-- Phase-60-Backfill traegt jede Bestandszeile eine eigene Person.
--
-- Block A wendet dafuer exakt dieselbe Gewinner-Regel an wie der Laufzeitpfad
-- in `dao_impl_sqlite/src/sales_person.rs`: die Sales Person behaelt bzw.
-- erzeugt ihre Identitaet und der Benutzer wandert auf sie (D-61-03); ein durch
-- ein Re-Link verdraengter Benutzer bekommt eine frische eigene Person
-- (D-61-05). Block B prueft sich anschliessend selbst und bricht ab, wenn
-- danach eine Verknuepfung ohne uebereinstimmendes `person_id`-Gegenstueck
-- bliebe — sonst ginge sie in Block C still verloren.
--
-- D-61-07: Block C loest die Tabelle auf. Es gibt KEINE Down-Migration und
-- keine Hilfstabelle mit Altzustand; der Wiederherstellungspfad ist die vor dem
-- Deploy gezogene Kopie der SQLite-Datei (identisch zu D-60-09).
--
-- Phase 60 und Phase 61 gehen gemeinsam live. Im Regelfall ist Block A damit
-- ein No-Op — der Nachzug existiert fuer den Fall, dass zwischen beiden
-- Migrationen doch Betrieb stattgefunden hat.
--
-- Anders als `ALTER TABLE ... ADD COLUMN` in Phase 60 ist `DROP TABLE IF
-- EXISTS` von sich aus mehrfach ausfuehrbar. Diese Datei braucht deshalb
-- KEINEN Marker-Split wie `20260916000000_create-person-backfill.sql`.

-- ─── Block A — Nachzieh-Backfill (D-61-06) ─────────────────────────────────
--
-- Sieben Anweisungen, deren Reihenfolge Teil des Vertrags ist:
--   A1/A2  Identitaet fuer unverknuepfte Zeilen (Idempotenz-Key `person_id IS
--          NULL`, unveraendert aus Phase 60 uebernommen).
--   A3     Entkopplung der im Fenster verdraengten Benutzer. Laeuft ZWINGEND
--          vor A5 — sonst kollidiert die Verschmelzung mit dem vollstaendigen
--          `user_person_id_unique_idx`.
--   A4     Identitaet fuer verknuepfte Sales Persons ohne `person_id`.
--   A5     Verschmelzung jeder lebenden Verknuepfung auf die Person der Sales
--          Person (D-61-03).
--   A6     Fuer jeden jetzt referenzierten Wert genau eine `person`-Zeile.
--   A7     Aufraeumung der Zeilen, die nach der Verschmelzung keinen Traeger
--          mehr haben. Laeuft ZWINGEND nach A6.
--
-- Die Gewinner-Richtung ist durchgehend dieselbe wie in Phase 60 und im
-- Laufzeitpfad: die Sales Person "gewinnt" die Zufallsgenerierung, der Benutzer
-- kopiert. Zwei unabhaengige Zufallswerte wuerden die beiden Seiten desselben
-- Paars auf zwei verschiedene Personen zeigen lassen (T-61-17).
--
-- Hier ist bewusst KEIN Soft-Delete-Filter auf der `deleted`-Spalte — dieselbe
-- Begruendung wie in Phase 60: soft-geloeschte Sales Persons brauchen ihre
-- Identitaet dauerhaft, weil write-once-Snapshots in
-- `billing_period_sales_person` an ihnen haengen (PERS-03). Der im Repo sonst
-- dominante DAO-Reflex gilt fuer Live-Queries, nicht fuer diesen Nachzug; ein
-- spaeterer Aufraeum-Pass darf ihn NICHT nachruesten.

-- A1) Unverknuepfte Sales Persons bekommen eine EIGENE zufaellige person_id.
UPDATE sales_person
SET person_id = randomblob(16)
WHERE person_id IS NULL
  AND id NOT IN (SELECT sales_person_id FROM sales_person_user);

-- A2) Unverknuepfte Benutzer bekommen eine EIGENE zufaellige person_id.
UPDATE user
SET person_id = randomblob(16)
WHERE person_id IS NULL
  AND name NOT IN (SELECT user_id FROM sales_person_user);

-- A3) Entkopplung: ein Benutzer, der die Person einer Sales Person traegt, mit
-- der er laut Verknuepfungstabelle NICHT (mehr) gepaart ist, wurde im Fenster
-- verdraengt — der alte Laufzeitpfad fuhr ein Re-Link als `discard` gefolgt von
-- `assign` und liess die alte `person_id` auf der Benutzerseite stehen.
-- D-61-05 sagt, was dann gilt: die Sales Person behaelt, der Benutzer bekommt
-- eine frische Identitaet.
--
-- Weil `sales_person.person_id` vollstaendig UNIQUE ist
-- (`sales_person_person_id_unique_idx`), liefert die innere Abfrage hoechstens
-- eine Sales Person — die Bedingung ist damit eindeutig. Diese Anweisung ist
-- der Grund, warum A5 spaeter nie in `UNIQUE constraint failed` laufen kann.
UPDATE user
SET person_id = randomblob(16)
WHERE person_id IS NOT NULL
  AND EXISTS (
    SELECT 1
    FROM sales_person sp
    WHERE sp.person_id = user.person_id
      AND NOT EXISTS (
        SELECT 1
        FROM sales_person_user spu
        WHERE spu.sales_person_id = sp.id
          AND spu.user_id = user.name
      )
  );

-- A4) Verknuepfte Paare, Sales-Person-Seite: diese Seite generiert.
UPDATE sales_person
SET person_id = randomblob(16)
WHERE person_id IS NULL
  AND id IN (SELECT sales_person_id FROM sales_person_user);

-- A5) Verschmelzung: JEDE lebende Verknuepfung zieht die Benutzerseite auf die
-- Person der Sales Person. Das deckt beide Fensterklassen ab — die Benutzerzeile
-- ohne Identitaet ebenso wie das auseinandergelaufene Bestandspaar.
--
-- Drei Details tragen die Korrektheit:
--   (a) `IS NOT` statt `!=` faengt den NULL-Fall mit und laesst bereits gleiche
--       Werte unangetastet — daran haengt die No-Op-Zusage aus dem Dateikopf.
--   (b) Der `IS NOT NULL`-Guard laesst eine Verknuepfungszeile mit fehlender
--       Elternzeile unberuehrt, statt dem Benutzer NULL zu schreiben. Block B
--       faengt genau diesen Zustand ab.
--   (c) `UNIQUE(sales_person_id)`/`UNIQUE(user_id)` auf `sales_person_user`
--       garantieren, dass die korrelierte Subquery hoechstens eine Zeile
--       liefert.
UPDATE user
SET person_id = (
    SELECT sp.person_id
    FROM sales_person_user spu
    JOIN sales_person sp ON sp.id = spu.sales_person_id
    WHERE spu.user_id = user.name
)
WHERE name IN (SELECT user_id FROM sales_person_user)
  AND (
    SELECT sp.person_id
    FROM sales_person_user spu
    JOIN sales_person sp ON sp.id = spu.sales_person_id
    WHERE spu.user_id = user.name
  ) IS NOT NULL
  AND person_id IS NOT (
    SELECT sp.person_id
    FROM sales_person_user spu
    JOIN sales_person sp ON sp.id = spu.sales_person_id
    WHERE spu.user_id = user.name
  );

-- A6) Fuer JEDEN jetzt referenzierten person_id-Wert genau EINE person-Zeile
-- anlegen. Das UNION dedupliziert: ein verknuepftes Paar liefert denselben Wert
-- aus beiden Quellen und erzeugt trotzdem nur eine Zeile. Idempotency key ist
-- der Ausschluss bereits angelegter Zeilen. Der update_process-Marker macht die
-- im Nachzug vergebenen Personen von den Phase-60-Backfill-Personen und von
-- spaeter regulaer angelegten unterscheidbar.
--
-- Zeitstempel-Format: bewusst strftime mit literalem 'T' statt
-- datetime('now') — die repo-weite Leseseite (Iso8601::DATE_TIME) lehnt das
-- Leerzeichen aus datetime('now') in beiden Parser-Zweigen ab. Identisch zur
-- Phase-60-Migration (dort CR-03 / I-13).
INSERT INTO person (id, created, update_process, update_version)
SELECT DISTINCT ids.person_id,
       strftime('%Y-%m-%dT%H:%M:%S.000000000Z', 'now'),
       'phase-61-drop-nachzug',
       randomblob(16)
FROM (
    SELECT person_id FROM sales_person WHERE person_id IS NOT NULL
    UNION
    SELECT person_id FROM user WHERE person_id IS NOT NULL
) ids
WHERE ids.person_id NOT IN (SELECT id FROM person);

-- A7) Traeger-Aufraeumung: die in A5 absorbierten und die in A3 verlassenen
-- person-Zeilen loeschen. Das ist exakt die zweite Haelfte der in
-- `docs/architecture/03-data-model.md` formulierten Invariante — jede
-- `person`-Zeile hat mindestens einen Traeger auf `sales_person` oder `user`.
--
-- Nach einem sauberen Phase-60-Backfill trifft diese Anweisung
-- definitionsgemaess nichts: dort traegt jede person-Zeile genau die Identitaet
-- einer bestehenden Sales Person oder eines bestehenden Benutzers. Die
-- No-Op-Zusage aus dem Dateikopf bleibt damit unberuehrt. Laeuft zwingend NACH
-- A6, sonst loeschte sie die gerade erst angelegten Zeilen nicht, sondern
-- schluege deren Anlage zu.
DELETE FROM person
WHERE NOT EXISTS (SELECT 1 FROM sales_person sp WHERE sp.person_id = person.id)
  AND NOT EXISTS (SELECT 1 FROM user u WHERE u.person_id = person.id);

-- ─── Block B — Selbst-Assertion (D-61-06) ──────────────────────────────────
--
-- Form analog zu D-60-07: SQLite kennt RAISE() nur in Trigger-Programmen, die
-- Wegwerf-Tabelle mit CHECK-Constraint ist das Aequivalent einer freistehenden
-- Assertion. Schlaegt der INSERT fehl, rollt sqlx die gesamte
-- Migrations-Transaktion zurueck — Block C laeuft dann nie, und die
-- Verknuepfung bleibt fuer die manuelle Bereinigung erhalten (T-61-16).
-- CREATE TEMP TABLE statt CREATE TABLE, damit selbst bei einem Fehler im DROP
-- nichts in der persistenten Datei zurueckbleibt; IF NOT EXISTS, damit ein
-- erneuter Lauf auf derselben Verbindung nicht an "table already exists"
-- scheitert und den eigentlichen Fehler maskiert.
CREATE TEMP TABLE IF NOT EXISTS __phase61_drop_assertion (
    ok INTEGER NOT NULL CHECK (ok = 1)
);

-- Geprueft wird genau die Bedingung, die der DROP gleich unwiderruflich macht:
-- es darf KEINE Verknuepfungszeile geben, deren Seiten nach Block A ohne
-- Identitaet oder mit zwei verschiedenen Identitaeten dastehen.
--
-- Den Fenster-Widerspruch (beide Seiten tragen je eine eigene Person) LOEST
-- Block A inzwischen selbst auf — Entkopplung in A3, Verschmelzung in A5,
-- Aufraeumung in A7. Diese Assertion steht deshalb fuer das, was Block A NICHT
-- reparieren kann: eine Verknuepfungszeile, deren Elternzeile fehlt (Herkunft:
-- Review-Befunde CR-01 und WR-05 der Phase 61).
--
-- Die aeusseren Joins sind zwingend: mit inneren Joins faellt genau die
-- verwaiste Zeile aus dem Ergebnis, die Assertion ginge gruen durch und Block C
-- wuerfe die Verknuepfung still weg. Die beiden globalen Deckungsterme sind aus
-- `20260916000000_create-person-backfill.sql` Block 4 uebernommen; dort steht im
-- Klartext, warum eine Pruefung ueber vorhandene Elternzeilen allein genau die
-- interessanten Zeilen verliert (Phase-60-Review CR-01).
INSERT INTO __phase61_drop_assertion (ok)
SELECT CASE
    WHEN (SELECT COUNT(*) FROM sales_person WHERE person_id IS NULL) = 0
     AND (SELECT COUNT(*) FROM user         WHERE person_id IS NULL) = 0
     AND NOT EXISTS (
        SELECT 1
        FROM sales_person_user spu
        LEFT JOIN sales_person sp ON sp.id = spu.sales_person_id
        LEFT JOIN user u ON u.name = spu.user_id
        WHERE sp.id IS NULL
           OR u.name IS NULL
           OR sp.person_id IS NULL
           OR u.person_id IS NULL
           OR sp.person_id != u.person_id
     )
    THEN 1 ELSE 0
END;

DROP TABLE __phase61_drop_assertion;

-- ─── Block C — Aufloesung (D-61-07) ────────────────────────────────────────
DROP TABLE IF EXISTS sales_person_user;
