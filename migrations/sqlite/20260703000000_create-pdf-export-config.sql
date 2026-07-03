-- Phase 48 (EXP-02/EXP-03, D-48-CONFIG): Single-Row-Konfigurationstabelle für
-- den Nextcloud-PDF-Export via WebDAV. Analog zu paid_limit_config /
-- holiday_stichtag_config: eine feste, in der Migration seedete Zeile mit
-- einer bekannten UUID als PK. Alle admin-editierbaren Werte + Status-Felder
-- des Scheduler-Laufs. Das Feature ist per Default deaktiviert (enabled=0),
-- der Admin füllt URL/User/Token/Ziel-Ordner nachträglich über die UI.
--
-- D-48-01: webdav_app_token wird KLARTEXT gespeichert (bewusste Ops-
-- Entscheidung, DB-File filesystem-geschützt — kein Feld-Encrypt in v2.2).
CREATE TABLE IF NOT EXISTS pdf_export_config (
    id BLOB NOT NULL PRIMARY KEY,
    enabled INTEGER NOT NULL DEFAULT 0,
    nextcloud_url TEXT,
    webdav_user TEXT,
    webdav_app_token TEXT,
    target_folder TEXT,
    weeks_horizon INTEGER NOT NULL DEFAULT 8,
    cron_schedule TEXT NOT NULL DEFAULT '0 6 * * 1',
    last_success_at TEXT,
    last_error_at TEXT,
    last_error_message TEXT,
    update_process TEXT NOT NULL,
    update_version BLOB NOT NULL
);

-- Seed die einzige Zeile mit fixer UUID (…0000048) und enabled=0.
-- URL/User/Token/Zielordner bleiben NULL bis der Admin die UI benutzt.
INSERT OR IGNORE INTO pdf_export_config (
    id, enabled, weeks_horizon, cron_schedule, update_process, update_version
) VALUES (
    X'00000000000000000000000000000048',
    0,
    8,
    '0 6 * * 1',
    'phase-48-migration',
    X'00000000000000000000000000000048'
);
