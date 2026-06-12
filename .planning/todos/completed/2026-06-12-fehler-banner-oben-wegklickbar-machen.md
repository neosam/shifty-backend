---
created: 2026-06-12 18:35
title: Fehler-Banner oben auf der Seite wegklickbar machen (Dismiss/Close)
area: frontend
files:
  - shifty-dioxus/src/component/error_view.rs (ErrorView — Banner ohne Close-Button)
  - shifty-dioxus/src/service/error.rs (ErrorStore/ErrorAction — nur SetError, kein Clear)
  - shifty-dioxus/src/error.rs (error_handler / result_handler — Fehler-Eintragspunkte)
---

## Problem

Wenn ein Fehler auftritt, wird er oben auf der Seite als Banner angezeigt
(`<ErrorView>` rendert `ERROR_STORE.error`). Dieser Banner **bleibt für immer
stehen** — der User kann ihn nicht wegklicken.

Ursache:
- `ErrorView` (`error_view.rs`) rendert nur das `error-message`-Div, hat aber
  **keinen Close-/Dismiss-Button**.
- Der `ErrorStore` (`service/error.rs`) kennt nur `ErrorAction::SetError` — es
  gibt **keine `Clear`/`Dismiss`-Action**, die `error` wieder auf `None` setzt.
  Ein einmal gesetzter Fehler wird also nie zurückgesetzt, höchstens vom
  nächsten Fehler überschrieben (Sonderfall: bei 401/UNAUTHORIZED macht
  `error_handler` einen Page-Reload, sonst bleibt er hängen).

Erwartet: Der User soll den Fehler-Banner wieder wegklicken können (X-Button /
Schließen), wonach `ERROR_STORE.error` auf `None` gesetzt wird und der Banner
verschwindet.

## Solution

TBD — kleiner Frontend-Fix:
1. `ErrorStore`/`ErrorAction` um eine `ClearError`-Variante erweitern (oder
   direkt `*ERROR_STORE.write() = ErrorStore::default()` im Click-Handler), die
   `error` auf `None` setzt.
2. In `ErrorView` einen Close-Button (X) rendern, dessen `onclick` den Fehler
   löscht.
3. Optional: Banner-Styling (`.error-view`) um Dismiss-Affordance ergänzen.

Klein (~Quick-Task). Tests: Frontend ist WASM — mind. WASM-Build-Gate
(`cargo build --target wasm32-unknown-unknown` in shifty-dioxus/) grün, ggf.
ein kleiner Logik-Test für die Clear-Action auf dem `ErrorStore`.

## Resolution (2026-06-12)

Umgesetzt:
1. `ErrorStore::cleared()`-Helper (no-error-State) + `ErrorAction::ClearError`-Variante
   in `service/error.rs`.
2. `ErrorView` rendert jetzt einen `×`-Dismiss-Button (`.error-dismiss`), dessen
   `onclick` `*ERROR_STORE.write() = ErrorStore::cleared()` setzt → Banner verschwindet.
   aria-label/title via neuem i18n-Key `ErrorBannerDismiss` (De/En/Cs).
3. Banner-Styling in `assets/main.css` (`.error-view` Flex-Row, `.error-message`,
   `.error-dismiss` Hover) — X sitzt rechts.

Tests: 2 neue Unit-Tests (`service::error::tests::cleared_has_no_error`,
`dismiss_replaces_existing_error_with_none`) grün; volle error-Suite 21 passed;
WASM-Build-Gate grün.
