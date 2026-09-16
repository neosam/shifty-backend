-- Phase 60 (PERS-01 / PERS-02 / PERS-03 / PERS-04 / PERS-06) — Person-Identitaet
-- als Elterntabelle ueber sales_person und user. Konsumenten (PersonDao,
-- Selbst-Zugriffs-Pruefung ueber person_id-Joins) folgen ab Phase 61; diese
-- Migration ist reine Schema- + Backfill-Arbeit ohne DAO-/Service-Eingriff
-- (D-60-10).
--
-- D-60-04: person traegt ausschliesslich Identitaet plus die 2026er
-- Audit-Spalten — KEINE deleted-Spalte, KEINE Namensspalten (PERS-F-01 ist
-- Future-Scope). In v2.7 wird keine Person geloescht.
--
-- D-60-01: Die beiden UNIQUE-Indizes auf person_id sind bewusst VOLLSTAENDIG
-- und nicht partiell. Es ist also bewusst KEIN "WHERE deleted IS NULL" gesetzt —
-- soft-geloeschte Sales Persons behalten ihre Person-Identitaet dauerhaft
-- (Produktentscheidung, siehe 60-CONTEXT.md D-60-01). NICHT beim naechsten
-- Aufraeum-Pass "reparieren": eine spaetere Umstellung auf einen partiellen
-- Index wuerde bereits vergebene Identitaeten neu binden und historische
-- billing_period_sales_person-Snapshots einer anderen Person zuordnen.
--
-- D-60-08: Der Backfill-Abschnitt zwischen den beiden BACKFILL-Markern ist
-- idempotent und wird vom Integrationstest service_impl::test::person_backfill
-- ein zweites Mal ausgefuehrt. Die Markerzeilen sind Teil des Vertrags — wer
-- sie umbenennt, schaltet den Apply-twice-Nachweis stumm ab.
--
-- D-60-09: Es gibt keine Down-Migration und keine Hilfstabelle mit Altzustand.
-- Der Wiederherstellungspfad ist die vor dem Deploy gezogene Kopie der
-- SQLite-Datei.

CREATE TABLE IF NOT EXISTS person (
    id blob(16) NOT NULL PRIMARY KEY,
    created TEXT NOT NULL,
    update_process TEXT NOT NULL,
    update_version blob(16) NOT NULL
);

-- D-60-03: person_id bleibt dauerhaft nullable und bekommt KEINEN DEFAULT.
-- Eine Person darf einen Benutzer ODER eine Sales Person ODER beides haben;
-- reine HR-/Admin-Accounts ohne Sales Person und Freiwillige ohne Login sind
-- ausdruecklich erlaubte Dauerzustaende, kein Uebergang.
ALTER TABLE sales_person ADD COLUMN person_id blob(16);
ALTER TABLE user ADD COLUMN person_id blob(16);

-- D-60-05: SQLite erlaubt bei ALTER TABLE ... ADD COLUMN kein UNIQUE, deshalb
-- kommt die 1:1-Garantie (PERS-04) als separater Index. Sie wird von der
-- Datenbank durchgesetzt, nicht von der Applikation.
-- D-60-01/D-60-02: vollstaendig, nicht partiell — siehe Kopfkommentar.
CREATE UNIQUE INDEX IF NOT EXISTS sales_person_person_id_unique_idx
    ON sales_person (person_id);

CREATE UNIQUE INDEX IF NOT EXISTS user_person_id_unique_idx
    ON user (person_id);

-- BACKFILL START
-- Idempotency key: person_id IS NULL. Re-run skips already-linked rows.
-- 3a) Unverknuepfte Sales Persons bekommen eine EIGENE zufaellige person_id.
-- Hier ist bewusst KEIN Soft-Delete-Filter gesetzt: der Backfill muss jeder
-- historischen Zeile eine Identitaet geben, sonst loesen write-once-Snapshots
-- in billing_period_sales_person spaeter ins Leere (PERS-03). Der im Repo
-- sonst dominante DAO-Reflex gilt fuer Live-Queries, nicht fuer diesen
-- einmaligen Backfill — ein spaeterer Aufraeum-Pass darf den Filter NICHT
-- nachruesten.
UPDATE sales_person
SET person_id = randomblob(16)
WHERE person_id IS NULL
  AND id NOT IN (SELECT sales_person_id FROM sales_person_user);

-- Idempotency key: person_id IS NULL. Re-run skips already-linked rows.
-- 3b) Unverknuepfte Benutzer bekommen eine EIGENE zufaellige person_id.
UPDATE user
SET person_id = randomblob(16)
WHERE person_id IS NULL
  AND name NOT IN (SELECT user_id FROM sales_person_user);

-- Idempotency key: person_id IS NULL. Re-run skips already-linked rows.
-- 3c) Verknuepfte Paare, Sales-Person-Seite: diese Seite "gewinnt" die
-- Zufallsgenerierung fuer das Paar. Auch hier bewusst ohne Soft-Delete-Filter.
UPDATE sales_person
SET person_id = randomblob(16)
WHERE person_id IS NULL
  AND id IN (SELECT sales_person_id FROM sales_person_user);

-- Idempotency key: person_id IS NULL. Re-run skips already-linked rows.
-- 3d) Verknuepfte Paare, Benutzer-Seite: KOPIERT den in 3c vergebenen Wert
-- ueber eine korrelierte Subquery. Hier wird bewusst NICHT neu gewuerfelt —
-- zwei unabhaengige Zufallswerte wuerden die beiden Seiten desselben Paars auf
-- zwei verschiedene Personen zeigen lassen und Success Criterion 1 sofort
-- brechen. Die Subquery liefert dank UNIQUE(sales_person_id)/UNIQUE(user_id)
-- auf sales_person_user garantiert hoechstens eine Zeile. Reihenfolge ist
-- zwingend: 3d laeuft nach 3c.
UPDATE user
SET person_id = (
    SELECT sp.person_id
    FROM sales_person_user spu
    JOIN sales_person sp ON sp.id = spu.sales_person_id
    WHERE spu.user_id = user.name
)
WHERE person_id IS NULL
  AND name IN (SELECT user_id FROM sales_person_user);

-- Idempotency key: Ausschluss bereits angelegter person-Zeilen
-- (ids.person_id NOT IN (SELECT id FROM person)). Re-run skips existing rows.
-- 3e) Fuer JEDEN jetzt referenzierten person_id-Wert genau EINE person-Zeile
-- anlegen. Das UNION dedupliziert: ein verknuepftes Paar liefert denselben Wert
-- aus beiden Quellen und erzeugt trotzdem nur eine Zeile — nur dadurch haelt
-- die Zaehlprobe weiter unten. Laeuft nach allen vier vorherigen Bloecken.
-- Der update_process-Marker macht die migrierten Personen dauerhaft von
-- spaeter regulaer angelegten unterscheidbar.
INSERT INTO person (id, created, update_process, update_version)
SELECT DISTINCT ids.person_id, datetime('now'), 'phase-60-person-backfill', randomblob(16)
FROM (
    SELECT person_id FROM sales_person WHERE person_id IS NOT NULL
    UNION
    SELECT person_id FROM user WHERE person_id IS NOT NULL
) ids
WHERE ids.person_id NOT IN (SELECT id FROM person);

-- Block 4 — Selbst-Verifikation (D-60-07). Bricht die Migration (und damit den
-- Server-Start) ab, wenn die Zaehlprobe aus Success Criterion 1 nicht aufgeht:
-- COUNT(person) = COUNT(sales_person) + COUNT(user) - COUNT(sales_person_user).
-- SQLite kennt RAISE() nur in Trigger-Programmen; ein Trigger wird hier
-- ausdruecklich NICHT angelegt (D-60-10). Die Wegwerf-Tabelle mit
-- CHECK-Constraint ist das Aequivalent einer freistehenden Assertion: schlaegt
-- der INSERT fehl, rollt sqlx die gesamte Migrations-Transaktion zurueck,
-- statt einen halb ausgefuehrten Backfill zu committen. CREATE TEMP TABLE
-- statt CREATE TABLE, damit selbst bei einem Fehler im DROP nichts in der
-- persistenten Datei zurueckbleibt. Dieser Block liegt INNERHALB der Marker,
-- damit der Apply-twice-Nachweis (D-60-08) auch den zweiten Lauf gegen die
-- Assertion fahren muss.
CREATE TEMP TABLE __phase60_backfill_assertion (
    ok INTEGER NOT NULL CHECK (ok = 1)
);

INSERT INTO __phase60_backfill_assertion (ok)
SELECT CASE
    WHEN (SELECT COUNT(*) FROM person)
       = (SELECT COUNT(*) FROM sales_person)
       + (SELECT COUNT(*) FROM user)
       - (SELECT COUNT(*) FROM sales_person_user)
    THEN 1 ELSE 0
END;

DROP TABLE __phase60_backfill_assertion;
-- BACKFILL END
