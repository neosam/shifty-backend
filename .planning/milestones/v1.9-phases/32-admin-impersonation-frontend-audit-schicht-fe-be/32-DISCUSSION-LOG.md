# Phase 32: Admin-Impersonation FE + Audit (FE+BE) - Discussion Log

> **Audit trail only.** Decisions are captured in CONTEXT.md.

**Date:** 2026-06-29
**Phase:** 32-Admin-Impersonation Frontend + Audit-Schicht (FE+BE)
**Areas discussed:** Audit-Mechanik, Banner-Identität

---

## Audit-Mechanik (IMP-03 → D-32-01)

| Option | Description | Selected |
|--------|-------------|----------|
| Zentrale Middleware (alle Writes) | Tower-Layer nach context_extractor loggt jede mutierende Anfrage (POST/PUT/PATCH/DELETE) mit real_user+acting_as+method+path, wenn RealUser gesetzt; + Start/Stop-Audit. Kein Blast-Radius, garantiert keine Schreib-Aktion unattribuiert. | ✓ |
| Nur Start/Stop-Audit | Nur bei Start/Stop loggen; Writes nur per Korrelation attribuiert. | |
| Pro Write-Handler | tracing in jeden mutierenden Handler; großer Blast-Radius. | |

**User's choice:** Zentrale Middleware (→ D-32-01)
**Notes:** Code-Scout bestätigte: context_extractor (2 cfg-Varianten) ist der RealUser-Inject-
Punkt; Layer in rest/src/lib.rs nach context_extractor. Admin-Gate gegen echte session.user_id
ist bereits im Code (D-32-02 nur zu dokumentieren).

---

## Banner-Identität (D-32-03)

| Option | Description | Selected |
|--------|-------------|----------|
| Username (user_id) direkt | Banner zeigt user_id aus ImpersonateTO; null BE-Änderung. | ✓ |
| Aufgelöster Anzeige-Name | ImpersonateTO um display_name erweitern (BE löst user_id→SalesPerson-Name auf). | |

**User's choice:** Username (user_id) direkt (→ D-32-03)
**Notes:** ImpersonateTO liefert user_id bereits; Friendly-Name deferred.

## Claude's Discretion
- Modul-Ort RealUser; Middleware-Name/-Reihenfolge; FE-Service-Store-Struktur; Banner-Mount-
  Punkt; welche Stores bei Start/Stop reloaded werden; user_id-Gewinnung aus Personenliste.

## Deferred Ideas
- Friendly-Name im Banner; DB-Audit + Audit-UI; Auto-Timeout; Impersonation anderer Admins.
