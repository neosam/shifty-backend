-- Phase 61 (PERS-05) — Aufloesung der 1:1-Verknuepfungstabelle
-- `sales_person_user`. Ab hier traegt allein `person_id` auf `sales_person` und
-- `user` die Kopplung; alle lesenden und schreibenden Pfade wurden in den
-- Plaenen 61-03 und 61-04 darauf umgestellt.
--
-- D-61-06: Block A zieht Zeilen nach, die zwischen dem Phase-60- und dem
-- Phase-61-Deploy entstanden sind und deshalb noch kein `person_id` tragen
-- (D-60-10 laesst das ausdruecklich zu). Block B prueft sich anschliessend
-- selbst und bricht ab, wenn danach eine Verknuepfung ohne uebereinstimmendes
-- `person_id`-Gegenstueck bliebe — sonst ginge sie in Block C still verloren.
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
-- Idempotency key durchgehend: `person_id IS NULL`. Reihenfolge und
-- Gewinner-Logik sind identisch zu den Phase-60-Bloecken 3a-3d — die Sales
-- Person "gewinnt" die Zufallsgenerierung, der Benutzer kopiert. Zwei
-- unabhaengige Zufallswerte wuerden die beiden Seiten desselben Paars auf zwei
-- verschiedene Personen zeigen lassen (T-61-17).
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

-- A3) Verknuepfte Paare, Sales-Person-Seite: diese Seite generiert.
UPDATE sales_person
SET person_id = randomblob(16)
WHERE person_id IS NULL
  AND id IN (SELECT sales_person_id FROM sales_person_user);

-- A4) Verknuepfte Paare, Benutzer-Seite: KOPIERT den in A3 vergebenen Wert
-- ueber eine korrelierte Subquery. Die Subquery liefert dank
-- UNIQUE(sales_person_id)/UNIQUE(user_id) auf sales_person_user garantiert
-- hoechstens eine Zeile. Reihenfolge ist zwingend: A4 laeuft nach A3.
UPDATE user
SET person_id = (
    SELECT sp.person_id
    FROM sales_person_user spu
    JOIN sales_person sp ON sp.id = spu.sales_person_id
    WHERE spu.user_id = user.name
)
WHERE person_id IS NULL
  AND name IN (SELECT user_id FROM sales_person_user);

-- A5) Fuer JEDEN jetzt referenzierten person_id-Wert genau EINE person-Zeile
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
-- Identitaet oder mit zwei verschiedenen Identitaeten dastehen. Ein Widerspruch
-- entsteht nicht durch Block A (dessen Idempotency key ueberspringt belegte
-- Zeilen), sondern durch manuelle Eingriffe oder Restores — genau dort, wo auch
-- die Pre-Flight-Query CR-02 der Phase 60 ansetzt.
INSERT INTO __phase61_drop_assertion (ok)
SELECT CASE WHEN NOT EXISTS (
    SELECT 1
    FROM sales_person_user spu
    JOIN sales_person sp ON sp.id = spu.sales_person_id
    JOIN user u ON u.name = spu.user_id
    WHERE sp.person_id IS NULL
       OR u.person_id IS NULL
       OR sp.person_id != u.person_id
) THEN 1 ELSE 0 END;

DROP TABLE __phase61_drop_assertion;

-- ─── Block C — Aufloesung (D-61-07) ────────────────────────────────────────
DROP TABLE IF EXISTS sales_person_user;
