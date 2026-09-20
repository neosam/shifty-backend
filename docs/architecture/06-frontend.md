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

## Routes & pages

Route definitions live in `shifty-dioxus/src/router.rs`; every route has
exactly one file under `page/`.

| Route | Page | Note |
| --- | --- | --- |
| `/` | `home.rs` | |
| `/shiftplan/`, `/shiftplan/:year/:week` | `shiftplan.rs` | |
| `/weekly_overview/` | `weekly_overview.rs` | |
| `/employees/`, `/employees/:employee_id/` | `employees.rs`, `employee_details.rs` | master/detail |
| `/my_employee_details/` | `my_employee_details.rs` | self-service |
| `/my-shifts/` | `my_shifts.rs` | |
| `/absences/`, `/absences/:employee_id/` | `absences.rs` | |
| `/billing-periods/`, `/billing_period/:billing_period_id/` | `billing_periods.rs`, `billing_period_details.rs` | |
| `/user_management/` | `user_management.rs` | the single person directory since v2.7 Phase 62 |
| `/person/:person_id/` | `person_detail.rs` | **the** person detail route since v2.7 Phase 63 |
| `/custom_extra_hours/` | `custom_extra_hours_management.rs` | |
| `/text_templates/` | `text_template_management.rs` | |
| `/settings/` | `settings.rs` | |

**One detail route for all three shapes (v2.7 Phase 63, D-63-04).** The two
former facet-specific detail routes and their pages were removed **without
replacement** in the same phase — a linked person, a user-only account and a
sales-person-only row all land on `/person/:person_id/`, which renders the
sections that actually apply. There is deliberately **no** redirect from the
old routes: a redirect would keep two URL shapes alive that carry no meaning
any more, and the only links into them lived in this frontend.

**The `person_id` in the URL can change** when a sales person is added to a
user-only entry [D-61-03]. The page therefore navigates to the `person_id`
the backend *returned*, never to the one it sent. See
[F01 §4.7](../features/F01-employee-management.md).

## Decision logic belongs in pure functions

**[Convention, established in v2.7 Phase 63]** Whatever a page *decides* —
which sections appear, which confirmation text applies, whether an action is
the last of its kind — belongs in pure functions with direct unit tests, not
inside the `rsx!` body. `page/person_detail.rs` carries three of them
(`visible_sections`, `is_last_facet`, `delete_confirmation_copy`); the SSR
render test then checks a *second* time that the result actually reaches the
markup.

The reason is the environment, not taste: DOM rendering is only testable to a
limited degree headless in this frontend stack (see the loader-pattern edge
case below for one instance of it). A pure function is fully testable, an
`rsx!` branch is not. Everything the pure functions cannot prove is therefore
planned as a **manual pass** from the start rather than discovered as a gap at
the end.

A pure helper must stay free of authorization decisions — it decides about
*display*, never about *permission*. The gate lives in the backend service;
a frontend function that branched on privileges would be a second truth of
which the weaker one is the hole.

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

### Person administration calls (v2.7, Phase 63)

`api.rs` gained eight calls for the person detail view, each with a
same-named wrapper in `loader.rs`:

| `api.rs` | Endpoint |
| --- | --- |
| `get_person_detail` | `GET /person/{id}` |
| `get_person_by_sales_person` | `GET /person/by-sales-person/{id}` |
| `post_person_entry` | `POST /person` |
| `post_person_facet` | `POST /person/{id}/facet` |
| `delete_person_facet` | `DELETE /person/{id}/facet/{facet}` |
| `get_unlinked_sales_persons` | `GET /person/candidates/sales-persons` |
| `get_unlinked_users` | `GET /person/candidates/users` |
| `delete_sales_person` | `DELETE /sales-person/{id}` |

The last one is **not** a new endpoint. `DELETE /sales-person/{id}` has
existed in the backend for a long time and is HR-gated; until Phase 63 the
frontend simply had no caller for it [D-63-06]. The deletion now happens
where the user sees the sales person, instead of as a side effect somewhere
else.

**Three distinguishable responses — because the page renders three different
things.** The translation sits in `api.rs`, where the response is still open
and a 422 body can be read:

- **404 is a *state*, not an error.** `load_person_detail` maps it to
  `Ok(None)`; the page shows its not-found state instead of raising the
  central error overlay. The store therefore keeps "not found" and "not
  loaded yet" in separate fields.
- **409 is the occupied target** of a race — the candidate list hid it, the
  server still rejects it [D-61-04]. The page offers a reload rather than an
  error.
- **422 is a validation failure** whose body names the field.

One exception by necessity: `save_user_for_sales_person` gets its translation
in `loader.rs` (`classify_link_error`), because the underlying
`api::post_user_to_sales_person` is an unchanged Phase 61 path with a
`reqwest::Error` signature and several existing callers.

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

Affected phases: 28 and 49 forgot this; by v2.7 the count of shipped
dev-server 404s from a missing entry stands at three.

**Order matters when one prefix contains another.** The proxy matches
prefix-literally. `/person` (Phase 63) is a prefix of `/person-directory`
(Phase 62), so the generic entry has to stand **after** the more specific
one; in front of it, it would swallow the directory calls.

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
