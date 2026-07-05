# Frontend Architecture

The frontend lives in `shifty-backend/shifty-dioxus/` and is its own
Cargo workspace, compiled to WebAssembly.

## Tech Stack

- **Framework:** [Dioxus](https://dioxuslabs.com/) 0.6.x.
- **Language:** Rust → WASM.
- **CSS:** Tailwind, compiled to `assets/tailwind.css`.
- **Dev server:** `dx serve` with hot reload.
- **Deploy:** Static bundle in `dist/`, served by the backend or a
  reverse proxy.

## Directory Structure

```
shifty-dioxus/src/
├── main.rs            # Entry point
├── app.rs             # Root component + router setup
├── router.rs          # Route definitions
├── auth.rs            # Login/logout/session state
├── api.rs             # HTTP client against backend
├── loader.rs          # Async data loading pattern
├── base_types.rs      # Core types (e.g. wrappers around rest-types)
├── error.rs           # Frontend error handling
├── js.rs              # JS interop
├── page/              # One Rust file per route
├── component/         # Reusable UI building blocks
├── service/           # Frontend services (wrappers around API)
├── state/             # Signal-based global state
├── i18n/              # Translations (En, De, Cs)
└── tests/
```

## Fat Backend, Thin Client

**Core principle:** All business logic lives in the backend. The frontend:

- renders results,
- validates input **only** for UX feedback (not as the authority),
- sends requests, listens for responses,
- displays errors.

It computes **no** balance, no conflicts, no snapshot values.

Why: A second client (mobile app, CLI, automation script) should
never have to duplicate a domain rule. Everything that counts comes
from the backend via REST.

## API Coupling: `rest-types`

DTOs are consumed from the `rest-types` crate — the same source of
truth as the backend. Field renames in the backend force the frontend
build to fail (which is intended).

## Dev Proxy — `Dioxus.toml`

In dev mode `dx serve` runs on port 8080 while the backend listens on
port 3000. The HTTP proxy configuration in `shifty-dioxus/Dioxus.toml`
forwards API paths to the backend:

```toml
[[web.proxy]]
backend = "http://localhost:3000/api/..."
```

**Edge case (happened 2× for real):** When you create a new backend
endpoint that the frontend should reach, you MUST also add a
`[[web.proxy]]` entry. Without it you get a 404 from `dx serve` in dev
mode even though the backend is running. Prod still works (the static
bundle goes through the reverse proxy).

Affected phases: both 28 and 49 forgot this.

## dx CLI Version Pin

**Important:** shifty-dioxus requires dx CLI **0.6.x** (the `dioxus`
crate is pinned to 0.6.3). When nixpkgs rolls to 0.7.x, the app does
not start and the design is stripped. The pin is locked down in
`flake.nix`.

If you start the app locally and see a blank layout:

- Check `dx --version`.
- Make sure `Dioxus.toml` sets `style = "/assets/tailwind.css"`.
- Verify the Tailwind watcher is running and actually writing to
  `assets/tailwind.css`.

## State Management

Signal-based (`dioxus::signals`). Global state in `state/`: auth
state, current user, selected Sales Person, current calendar week.

## Loader Pattern

Async data loading uses a Loader trait that yields `Loading` /
`Loaded` / `Error`. Pages render the corresponding UI state.

**Edge case:** For programmatic date pickers (`<input type=date>`),
JS injects do not trigger Dioxus signals — submit buttons stay
inactive. For automation tests, prefer value verification via cargo
test instead of browser automation. See memory
`reference_dioxus_browser_test_date_inputs`.

## i18n

Three languages: **En, De, Cs**. Every new text needs entries in all
three locales.

Details: [`08-i18n.md`](./08-i18n.md).

## Frontend Testing

- WASM build gate: `cargo build --target wasm32-unknown-unknown` in
  the `shifty-dioxus/` directory.
- **Clippy:** Excluded from the backend CI clippy run (198 pre-existing
  lints). Must be run manually, and from the backend shell — the
  dioxus shell is broken with E0514.

## Verify Backend Roundtrip

**[Convention]** Frontend phases with a "backend already exists"
assumption MUST be verified end-to-end in the browser — the create
path is not the edit path. Phase 23 learned that `modify_slot`
dropped `max_paid_employees` even though `create_slot` set it
correctly. Only a browser test caught this.

## Related edge cases

See [`../domain/edge-cases.md#11-frontend-backend-kopplung`](../domain/edge-cases.md#11-frontend-backend-kopplung).
