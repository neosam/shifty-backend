-- v2.3.1 Hotfix: der PDF-Export-Scheduler benutzt intern
-- `tokio-cron-scheduler` 0.15 → `croner` 3.0, das crontab-Ausdrücke mit
-- `Seconds::Required` parst — also im 6-Feld-Format
-- (`sec min hour dom mon dow`). Die ursprüngliche Migration
-- `20260703000000_create-pdf-export-config.sql` seedete jedoch den
-- 5-Feld-DEFAULT `'0 6 * * 1'`. Sobald ein Admin `enabled=true` gesetzt hat,
-- brach `Job::new_async(...)` beim Reload mit einem Parse-Fehler ab; beim
-- Backend-Start eskalierte das bis in die `.expect(...)`-Panic in
-- `shifty_bin/src/main.rs`.
--
-- Diese Data-Fix-Migration normalisiert bestehende 5-Feld-Werte auf
-- 6-Feld, indem sie ein Sekunden-Feld ('0 ') voranstellt. Ein
-- crontab-Ausdruck hat genau vier Leerzeichen, wenn er 5 Felder hat;
-- diese Länge-basierte Erkennung ist robust gegen sonstige
-- SQLite-String-Beschränkungen und lässt bereits gültige 6/7-Feld-Werte
-- unverändert.
UPDATE pdf_export_config
SET cron_schedule = '0 ' || cron_schedule
WHERE (length(cron_schedule) - length(replace(cron_schedule, ' ', ''))) = 4;
