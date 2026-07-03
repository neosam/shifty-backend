---
created: 2026-07-03T00:00:00.000Z
title: Schmale Admin-Rolle mit User-Impersonation ("act as")
area: auth
files:
  - service/src/permission.rs
  - service_impl/src/permission.rs
  - rest/src/lib.rs
  - migrations/sqlite/20240426150045_user-roles.sql
---

## Problem

Aktuell ist "admin" eine Rolle, die faktisch god mode bedeutet (siehe
[[2026-05-08-admin-rolle-bekommt-alle-privilegien]] — Wunsch dort: admin hat
automatisch ALLE Privilegien). Nebenwirkung: Wer täglich als Admin eingeloggt
ist, sieht die Anwendung nie aus Sicht eines normalen Users, und jede
Alltagsaktion läuft mit Vollrechten. Für neosam (Owner + normaler
Mitarbeiter) unschön: er will primär als normaler Benutzer arbeiten und nur
für echte Admin-Tätigkeiten in die Admin-Rolle wechseln.

**Gewünschtes Modell:**

1. Eine dedizierte, sehr schmale Rolle (Arbeitsname `user_admin` / `impersonator`)
   mit exakt zwei Kompetenzen:
   - Benutzer/Rollen/Privilegien verwalten (die bestehenden user-management-
     Privilegien).
   - Als anderer Benutzer agieren (Impersonation / "act as").
2. Der Owner läuft im Alltag mit einem normalen User-Account (z. B. `sales`-
   Rolle) und hat separat einen Admin-Account, dessen Identität er nur bei
   Bedarf annimmt.

Zwei Umsetzungs-Achsen:

**Achse A — Rolle:** Neuer Privilege-Satz `user.manage` + `user.impersonate`
(oder ähnlich), an eine neue Rolle gebunden. Bestehende `admin`-Rolle bleibt
für "Vollzugriff auf Domain-Features"; die neue Rolle ist bewusst nicht `admin`.

**Achse B — Impersonation-Mechanik:** Wie wechselt man Identitäten?
- **B1 — Zwei Accounts, klassisches Login/Logout:** Simpel, kein Feature-Code
  nötig; nur Rolle A umsetzen. Nachteil: OIDC-Session-Wechsel ist umständlich.
- **B2 — In-App "act as user":** Admin (mit `user.impersonate`) wählt Ziel-User
  → Session-Token wird auf Ziel-User umgestellt (mit Audit-Trail + sichtbarem
  Banner "Du agierst als X"). Braucht Backend-Endpoint (`POST
  /admin/impersonate/{user_id}` → neues JWT/Cookie mit `sub=target_user`,
  `act=admin_user`), Frontend-Banner, Ausstiegs-Knopf, Audit-Log-Eintrag.

## Solution

**Empfohlen: Achse A jetzt, Achse B2 später.**

**Phase 1 (klein, schnell):**
- Zwei neue Privilegien: `user.manage`, `user.impersonate`.
- Neue Rolle `user_admin` mit genau diesen zwei Privilegien.
- Bestehende user-management-Endpoints (invitations, roles, privileges) gaten
  auf `user.manage` statt `admin`-Rolle.
- Migration: `user_admin`-Rolle anlegen, aber nicht automatisch zuweisen —
  Owner weist sich explizit zu.
- Wenn [[2026-05-08-admin-rolle-bekommt-alle-privilegien]] Option A (Code-
  Wildcard "admin = alles") umgesetzt wird, `user.impersonate` bewusst NICHT
  ins Wildcard einbeziehen — sonst ist der reguläre admin auch impersonator,
  und die Trennung ist wieder weg. Entweder Wildcard nur auf Domain-Privilegien
  (nicht auf `user.*`), oder explizite Allow-Liste.

**Phase 2 (später, wenn's weh tut):**
- Impersonation-Endpoint (B2) mit Audit-Trail (`impersonation_log`-Tabelle:
  wer, wen, wann, wie lange).
- Frontend-Banner + Exit-Knopf.
- OIDC-Kompatibilität prüfen (kann das aktuelle JWT-Setup einen `act`-Claim?).

## Offene Fragen

- Soll die neue Rolle wirklich "user_admin" heißen oder etwas eindeutigeres wie
  `iam_admin` / `identity_admin`?
- Muss `user.impersonate` per Config auf spezifische Ziel-Rollen limitierbar
  sein (z. B. nur "sales" impersonieren, nicht andere Admins), oder reicht
  "alle User"?
- Interaktion mit OIDC im Production-Modus: kann der IDP über einen zweiten
  Account laufen, oder braucht B2 zwingend eine Backend-Impersonation, weil der
  IDP-Login-Wechsel zu umständlich ist?

## Acceptance Criteria

- [ ] Neue Privilegien `user.manage`, `user.impersonate` in Migration.
- [ ] Neue Rolle `user_admin` (oder Alternativ-Name) mit genau diesen zwei.
- [ ] User-Management-Endpoints gaten auf `user.manage`.
- [ ] Test: User mit Rolle `user_admin` kann User verwalten, aber KEINE Domain-
  Endpoints (Shiftplan, Booking, Reports) aufrufen.
- [ ] Test: User mit Rolle `admin` (falls Wildcard aktiv) hat NICHT
  automatisch `user.impersonate`.
- [ ] Doku: wie Owner sich beide Accounts (normaler + user_admin) einrichtet.
- [ ] (Phase 2, separater Todo) Impersonation-Endpoint + Audit-Log + Banner.
