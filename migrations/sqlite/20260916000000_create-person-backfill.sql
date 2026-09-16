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
-- BACKFILL END
