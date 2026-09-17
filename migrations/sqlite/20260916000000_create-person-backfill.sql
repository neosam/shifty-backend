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
--
-- ZEITSTEMPEL-FORMAT (Plan 60-04 / CR-03): bewusst strftime statt
-- datetime('now'). datetime('now') liefert "YYYY-MM-DD HH:MM:SS" mit einem
-- LEERZEICHEN, und die repo-weite Leseseite (Iso8601::DATE_TIME, siehe
-- dao_impl_sqlite/src/billing_period.rs) verlangt ein literales 'T' — beide
-- Zweige der dortigen Parser-Kette (OffsetDateTime, dann PrimitiveDateTime)
-- lehnen das Leerzeichen ab. person.created ist write-once, der PersonDao aus
-- Phase 61 liest genau diese Zeilen. Das gewaehlte Muster wurde empirisch
-- gegen die Parser-Kette geprueft und ist das einzige der getesteten
-- Kandidaten, das in BEIDEN Zweigen parst; sqlite 'now' ist UTC, das 'Z' ist
-- also sachlich korrekt. Festgezurrt von I-13
-- (service_impl::test::person_backfill::person_created_is_iso8601_parsable).
INSERT INTO person (id, created, update_process, update_version)
SELECT DISTINCT ids.person_id,
       strftime('%Y-%m-%dT%H:%M:%S.000000000Z', 'now'),
       'phase-60-person-backfill',
       randomblob(16)
FROM (
    SELECT person_id FROM sales_person WHERE person_id IS NOT NULL
    UNION
    SELECT person_id FROM user WHERE person_id IS NOT NULL
) ids
WHERE ids.person_id NOT IN (SELECT id FROM person);

-- Block 4 — Selbst-Verifikation (D-60-07). Bricht die Migration (und damit den
-- Server-Start) ab, wenn Success Criterion 1 nach dem Backfill nicht gilt.
-- SQLite kennt RAISE() nur in Trigger-Programmen; ein Trigger wird hier
-- ausdruecklich NICHT angelegt (D-60-10). Die Wegwerf-Tabelle mit
-- CHECK-Constraint ist das Aequivalent einer freistehenden Assertion: schlaegt
-- der INSERT fehl, rollt sqlx die gesamte Migrations-Transaktion zurueck,
-- statt einen halb ausgefuehrten Backfill zu committen. CREATE TEMP TABLE
-- statt CREATE TABLE, damit selbst bei einem Fehler im DROP nichts in der
-- persistenten Datei zurueckbleibt. Dieser Block liegt INNERHALB der Marker,
-- damit der Apply-twice-Nachweis (D-60-08) auch den zweiten Lauf gegen die
-- Assertion fahren muss.
--
-- IF NOT EXISTS (Plan 60-04 / WR-03): schlaegt der INSERT unten fehl, stoppt
-- sqlx an dieser Anweisung und das DROP laeuft nie. Die TEMP-Tabelle ist
-- verbindungslokal und ueberlebt. Ohne den Guard scheitert ein erneuter Lauf auf
-- derselben Verbindung an "table already exists" und maskiert damit den
-- eigentlichen Fehler. Die Tabelle ist beim Wiedereintritt immer leer: die
-- einzige schreibende Anweisung ist der INSERT direkt darunter, und der schreibt
-- im Fehlerfall nichts.
CREATE TEMP TABLE IF NOT EXISTS __phase60_backfill_assertion (
    ok INTEGER NOT NULL CHECK (ok = 1)
);

-- Geprueft werden DREI Bedingungen. Die Zaehlarithmetik allein reicht nicht
-- (Plan 60-04 / CR-01): zeigt eine sales_person_user-Zeile auf eine nicht
-- aufloesbare sales_person, dann ueberspringt Block 3b den Benutzer und Block 3d
-- schreibt ihm per korrelierter Subquery NULL — die Zaehlprobe geht mit
-- 1 == 1 + 1 - 1 trotzdem auf, und ein user ohne Identitaet committet lautlos.
-- Die Zaehlprobe ist fuer "jede Zeile hat eine Identitaet" also weder notwendig
-- noch hinreichend; sie bleibt als Deduplizierungs-Nachweis fuer Block 3e stehen,
-- die Deckungspruefung kommt separat dazu.
--
-- Der so erkannte Datenzustand ist ueber die Applikation nicht erzeugbar: sqlx
-- setzt PRAGMA foreign_keys = ON auf jeder Verbindung
-- (sqlx-sqlite/src/options/mod.rs), und sales_person wird nur soft-geloescht.
-- Die Pruefung ist Haertung, kein Schutz vor einem bekannten Produktionspfad.
INSERT INTO __phase60_backfill_assertion (ok)
SELECT CASE
    WHEN (SELECT COUNT(*) FROM sales_person WHERE person_id IS NULL) = 0
     AND (SELECT COUNT(*) FROM user WHERE person_id IS NULL) = 0
     AND (SELECT COUNT(*) FROM person)
       = (SELECT COUNT(*) FROM sales_person)
       + (SELECT COUNT(*) FROM user)
       - (SELECT COUNT(*) FROM sales_person_user)
    THEN 1 ELSE 0
END;

DROP TABLE __phase60_backfill_assertion;
-- BACKFILL END
