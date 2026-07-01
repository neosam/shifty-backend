---
created: 2026-07-01T15:57:00Z
title: Urlaub beantragen mit Genehmigungs-Workflow
area: absence
files:
  - service/src/absence.rs
  - service_impl/src/absence.rs
  - rest/src/absence.rs
  - rest-types/src/absence.rs
  - shifty-dioxus/src/page/absences.rs
---

## Problem

Aktuell werden Abwesenheiten/Urlaube (Absence-Modul) direkt eingetragen und sind sofort
wirksam. Es fehlt ein **Antrags-/Genehmigungs-Workflow**: Mitarbeiter sollen Urlaub
**beantragen** können, und diese Anträge müssen von einer berechtigten Person (z. B.
Admin/Vorgesetztem) erst **genehmigt** werden, bevor sie gültig/verbindlich sind.

Nutzen: klare Freigabekette, keine sofort wirksamen Urlaube ohne Kontrolle, Nachvollzieh-
barkeit wer wann genehmigt/abgelehnt hat.

## Solution

TBD — vor Umsetzung klären:
- **Status-Modell:** Absence bekommt einen Status (z. B. `requested` / `approved` /
  `rejected`, evtl. `cancelled`). Migration + `snapshot_schema_version`-Auswirkung prüfen,
  falls genehmigte Abwesenheiten in Reporting/Balance einfließen (nur `approved` zählt).
- **Berechtigungen:** Wer darf beantragen (self), wer genehmigen/ablehnen (Rolle/Privilege)?
  RBAC-Gate im `AbsenceService`.
- **Wirkung auf Berechnungen:** Nur genehmigte Anträge dürfen in Balance/Reporting/Kapazität
  einfließen — offene Anträge nicht (Doppelzählung/Leak vermeiden, vgl. `is_paid`-Gating).
- **Backend:** `AbsenceService`/DAO um Status + Genehmigungs-Operationen erweitern (approve/
  reject), REST-Endpunkte inkl. `#[utoipa::path]`, Transaktions-Pattern.
- **Frontend:** Antrags-Ansicht für Mitarbeiter + Genehmigungs-Ansicht (offene Anträge,
  approve/reject), Status-Anzeige, i18n (En/De/Cs).
- **Benachrichtigung (optional):** Antragsteller über Genehmigung/Ablehnung informieren?

Verwandt: [[2026-06-27-urlaub-fuer-freiwillige-eintragen]] (Absence für Freiwillige),
v1.0 Absence-Periods.
