-- Phase 1: range-based absence domain (additiv).
-- Strikt additiv: keine Änderungen an extra_hours, billing_period o.ä.
--
-- Recovery-Migration für Plan 03-06: Phase-1 hatte ursprünglich
-- `20260501162017_create-absence-period.sql` mit identischem Schema; die Datei
-- ging zwischen Phase-1-Worktree-Branch und main verloren (siehe
-- `.planning/phases/03-.../deferred-items.md` — "Phase-1-Migrations-Lücke").
-- Plan 03-06 stellt sie wieder her, weil Cross-Source-Integration-Tests
-- (Plan-01-Wave-5-Stubs) die Tabelle voraussetzen.
--
-- Schema verbatim aus `.planning/phases/01-absence-domain-foundation/01-00-PLAN.md`
-- Z. 134-164 (D-04, D-05, D-13 / SC4 / Pitfall-1).

CREATE TABLE absence_period (
    id              BLOB(16) NOT NULL PRIMARY KEY,
    logical_id      BLOB(16) NOT NULL,
    sales_person_id BLOB(16) NOT NULL,
    category        TEXT NOT NULL,
    from_date       TEXT NOT NULL,
    to_date         TEXT NOT NULL,
    description     TEXT,
    created         TEXT NOT NULL,
    deleted         TEXT,
    update_timestamp TEXT,
    update_process  TEXT NOT NULL,
    update_version  BLOB(16) NOT NULL,

    CHECK (to_date >= from_date),

    FOREIGN KEY (sales_person_id) REFERENCES sales_person(id)
);

CREATE UNIQUE INDEX idx_absence_period_logical_id_active
    ON absence_period(logical_id)
    WHERE deleted IS NULL;

CREATE INDEX idx_absence_period_sales_person_from
    ON absence_period(sales_person_id, from_date)
    WHERE deleted IS NULL;

CREATE INDEX idx_absence_period_self_overlap
    ON absence_period(sales_person_id, category, from_date)
    WHERE deleted IS NULL;
