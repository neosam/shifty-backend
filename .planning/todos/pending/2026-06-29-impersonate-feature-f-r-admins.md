---
created: 2026-06-29T12:48:02.144Z
title: Impersonate-Feature für Admins
area: auth
files:
  - service/src/permission.rs
  - service_impl/src/permission.rs
  - rest/src/lib.rs
---

## Problem

Admins brauchen die Möglichkeit, sich vorübergehend als ein anderer Benutzer
auszugeben („impersonate"), um dessen Sicht zu reproduzieren — typische
Support-/Debugging-Use-Cases: „Mitarbeiter X sieht seinen Report/seine
Abwesenheiten/sein Zeitkonto falsch, ich will genau das sehen, was er sieht".

Aktuell läuft die Authentifizierung über `Authentication<Context>`, das durch alle
Service-Calls gereicht wird (Dev: Mock-Admin; Prod: OIDC). Es gibt keinen Weg, die
effektive Identität temporär zu wechseln, ohne sich als der andere User einzuloggen.

## Solution

TBD — vor Planung zu klären:

- **Mechanismus:** Server-seitiger Impersonation-Context (Admin-Identität bleibt für
  Audit erhalten, „acting as" wird zusätzlich getragen) vs. reines Token-Swapping.
  Bevorzugt: `Authentication`/`Context` um ein optionales `impersonated_sales_person_id`
  erweitern, das nur gesetzt werden darf, wenn der echte Caller `admin`-Privileg hat.
- **Scope der Impersonation:** read-only (nur Ansichten) oder auch schreibend? Read-only
  ist deutlich risikoärmer für den ersten Wurf.
- **Permission-Gate:** Start/Stop der Impersonation strikt `admin`-gated; jede
  impersonierte Aktion sollte im Audit/Log die echte Admin-Identität mitführen
  (kein Identitäts-Verlust für Nachvollziehbarkeit).
- **Frontend:** sichtbarer „Du agierst als <Name> — Impersonation beenden"-Banner
  (nicht-blockierend, vgl. Banner-statt-Dialog-Konvention), Start z.B. aus der
  HR-/Admin-Personenliste. i18n de/en/cs.
- **Sicherheit:** keine Privilege-Eskalation — impersonierter User darf nie mehr
  Rechte bekommen als er selbst hat; Admin-Rechte dürfen nicht „durchschlagen".

Berührt vermutlich: `Authentication`/`Context`-Typen + Permission-Service
(`service/src/permission.rs`, `service_impl/src/permission.rs`), REST-Auth-Layer
(`rest/src/lib.rs`), und einen neuen HR-/Admin-gated Endpoint zum Setzen/Aufheben.
Eigenes Milestone-Thema (off-theme zu allem bisherigen) — Kandidat für `/gsd-new-milestone`
oder Backlog-Promotion.
