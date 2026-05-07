---
created: 2026-05-07T10:50:00
title: "Review: list_user_invitations gibt bei Parse-Fehler stilles leeres Array zurück"
area: frontend / api
files:
  - shifty-dioxus/src/api.rs:1127-1171
---

## Beobachtung

Aus Frontend-Codebase-Map (`.planning/codebase/frontend/CONCERNS.md` §2),
nicht reproduziert. **Bitte bestätigen oder als false positive markieren.**

`list_user_invitations` in `shifty-dioxus/src/api.rs:1127-1171` macht bei
Deserialization-Fehler folgendes (Zeilen 1163–1169):

```rust
Err(e) => {
    tracing::error!("Failed to deserialize invitations: {}", e);
    tracing::error!("Response text was: {}", response_text);
    // For now, return empty array but log the error properly
    // TODO: Find a better way to convert serde error to reqwest error
    Ok(Rc::new([]))
}
```

Der Author hat sich selbst einen TODO-Kommentar hinterlegt — also schon
beim Schreiben als suboptimal erkannt. Erfolg-Pfad gibt eine Liste zurück,
Fehler-Pfad gibt **leise eine leere Liste** zurück. Es wird zwar via
`tracing::error!` geloggt, aber der Caller erhält ein syntaktisch
gültiges `Ok` und die UI zeigt schlicht "keine Invitations" — ohne
Fehler-Banner, ohne Toast, ohne Hinweis dass etwas schiefging.

## Warum das jetzt auffällt

Das ist exakt der User-sichtbare Failure-Modus der `rest-types`-Drift
(siehe `.planning/codebase/frontend/CONCERNS.md` §1):

- Backend fügt neue Variante zu `InvitationStatus` hinzu (z. B. eine
  neue Status-Enum-Variante)
- Frontend-`rest-types`-Fork kennt sie nicht
- Deserialization schlägt fehl → leeres Array → UI sagt "keine Einladungen"
- Operator denkt "Bug, Backend liefert nichts" und sucht im falschen Ort

Die Drift-Konsolidierung wird das Risiko reduzieren, aber das Pattern
bleibt anfällig für jede künftige API-Erweiterung, die das Frontend
nicht mitbekommt.

## Frage an dich

1. Ist dir das Verhalten bekannt und intentional? (z. B. „Lieber leere
   Liste als kaputte UI während Frontend-Backend-Lag")
2. Falls nein: User-feedback-fähig machen? Optionen:
   - Echten Fehler propagieren (bricht aktuelle UI bis ein Error-Boundary
     existiert)
   - `Result<Rc<[InvitationResponse]>, FetchError>` mit eigenem Error-Typ,
     UI rendert "Daten konnten nicht geladen werden" mit Retry
   - Toast/Banner aus einem zentralen Error-Channel

Andere Stellen im selben `api.rs` checken — wenn das ein Pattern ist,
lohnt sich ein eigener Plan dafür.

## Nicht in Scope dieses Todos

- `api.rs`-Monolith insgesamt aufzuteilen (1269 Zeilen) — separates Thema,
  größerer Refactor.
