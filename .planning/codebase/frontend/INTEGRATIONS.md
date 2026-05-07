# External Integrations

**Analysis Date:** 2026-05-07

This document describes the **frontend** crate (`shifty-dioxus`) at
`/home/neosam/programming/rust/projects/shifty/shifty-dioxus/`. All file paths
in this document are relative to that directory.

## Architectural Overview

The frontend is a single-page WASM application that talks to **exactly one**
external service at runtime: the `shifty-backend` REST API. There are no
direct integrations with payment providers, third-party analytics, error
trackers, or storage services. State is held in Dioxus `GlobalSignal`s; the
only persistence outside the backend is `localStorage["shifty-theme"]`
(theme preference, set by inline JS in `index.html` lines 12–22).

```
┌────────────────────────────────────────────────────────────────┐
│  shifty-dioxus (WASM, browser)                                 │
│  ┌────────┐  ┌──────────────────┐  ┌─────────────────────────┐ │
│  │ pages/ │→ │ service/ (state) │→ │ src/api.rs (reqwest)    │ │
│  │ comps/ │  │  GlobalSignals   │  │  + rest_types::*TO DTOs │ │
│  └────────┘  └──────────────────┘  └────────────┬────────────┘ │
└─────────────────────────────────────────────────┼──────────────┘
                                                  │ HTTP/JSON
                                                  ▼
                              shifty-backend  http://localhost:3000
                                       (Axum, OpenAPI utoipa)
```

## Backend REST API (sole external integration)

### Base URL & Configuration

- The base URL lives at `config.backend` (`Rc<str>`), populated by
  `api::load_config()` from `assets/config.json`. Default committed value:
  `"http://localhost:8080"` (the dev server's own origin, which proxies the
  enumerated paths to `http://localhost:3000`).
- Config schema: `src/state/config.rs:18-28` (`Config`).
- Auth-info bootstrap uses a separate URL form: `fetch_auth_info` in
  `src/api.rs:22-32` is called with the backend URL and hits `/auth-info`.
- All other API helpers receive a cloned `Config` and `format!()` URLs as
  `{config.backend}/<path>`.

### HTTP Client

- `reqwest::get(url)` for plain GETs and `reqwest::Client::new()` for
  `.post()` / `.put()` / `.delete()` with `.json(&payload)`. No bearer-token
  injection — the browser carries cookies automatically (session cookie set
  by the backend OIDC flow).
- Error handling: helpers return `Result<T, reqwest::Error>` (most) or
  `Result<T, ShiftyError>` (where 409 conflict needs distinct handling, e.g.
  `update_extra_hour` at `src/api.rs:452-468`).
- The global error handler at `src/error.rs:18-33` reloads the page on HTTP
  401, which is what triggers re-authentication.

### Endpoints Consumed

The frontend exercises the following backend prefixes. Each row lists the
function in `src/api.rs` (line numbers approximate) along with the methods
hit. The full URL is always `{config.backend}/<path>`.

| Backend prefix | `src/api.rs` callers | Methods |
|----------------|----------------------|---------|
| `/auth-info` | `fetch_auth_info` (L22) | GET |
| `/version` | `get_version` (L470) | GET |
| `/shiftplan-catalog` | `get_all_shiftplans` (L70), `create_shiftplan` (L80), `update_shiftplan` (L102), `delete_shiftplan` (L116) | GET / POST / PUT / DELETE |
| `/shiftplan-edit/slot/{year}/{week}[/{slot_id}]` | `update_slot` (L136), `delete_slot_from` (L163) | PUT / DELETE |
| `/shiftplan-edit/vacation` | `add_vacation` (L760) | PUT |
| `/shiftplan-info/{shiftplan_id}/{year}/{week}` | `get_shiftplan_week` (L781) | GET |
| `/shiftplan-info/day/{year}/{week}/{day}` | `get_shiftplan_day` (L799) | GET |
| `/slot[/{id}]`, `/slot/week/{year}/{week}/{shiftplan_id}` | `get_slots` (L55), `get_slot` (L126), `post_slot` (L150) | GET / POST |
| `/booking[/{id}]`, `/booking/week/{year}/{week}`, `/booking/copy?...` | `get_bookings_for_week` (L181), `add_booking` (L195), `remove_booking` (L225), `copy_week` (L235) | GET / POST / DELETE |
| `/booking-information/conflicts/for-week/{year}/{week}`, `/booking-information/weekly-resource-report/{year}` | `get_booking_conflicts_for_week` (L651), `get_weekly_overview` (L680) | GET |
| `/booking-log/{year}/{week}` | `get_booking_log` (L666) | GET |
| `/sales-person[/{id}]`, `/sales-person/current`, `/sales-person/by-user/{username}` | `get_sales_persons` (L251), `get_current_sales_person` (L261), `get_sales_person` (L273), `put_sales_person` (L286), `post_sales_person` (L303), `get_sales_person_by_user` (L939) | GET / POST / PUT |
| `/sales-person/{id}/user`, `/sales-person/{id}/unavailable`, `/sales-person/unavailable[/{id}]` | `get_user_for_sales_person` (L317), `post_user_to_sales_person` (L330), `delete_user_from_sales_person` (L344), `get_unavailable_sales_person_days_for_week` (L480), `create_unavailable_sales_person_day` (L498), `delete_unavailable_sales_person_day` (L526) | GET / POST / DELETE |
| `/sales-person-shiftplan/{id}/shiftplans`, `/sales-person-shiftplan/by-shiftplan/{id}` | `get_shiftplan_assignments` (L1203), `set_shiftplan_assignments` (L1219), `get_bookable_sales_persons` (L1236) | GET / PUT |
| `/extra-hours[/{id}]`, `/extra-hours/by-sales-person/{id}?year=&until_week=` | `add_extra_hour` (L392), `get_extra_hours_for_year` (L424), `delete_extra_hour` (L442), `update_extra_hour` (L452) | GET / POST / PUT / DELETE |
| `/custom-extra-hours[/{id}]`, `/custom-extra-hours/by-sales-person/{id}` | `get_custom_extra_hours_by_sales_person` (L826), `post_custom_extra_hours` (L842), `put_custom_extra_hours` (L855), `delete_custom_extra_hours` (L871) | GET / POST / PUT / DELETE |
| `/working-hours[/{id}]`, `/working-hours/for-sales-person/{id}` | `get_employee_work_details_for_sales_person` (L706), `post_employee_work_details` (L720), `put_employee_work_details` (L732), `delete_employee_work_details` (L748) | GET / POST / PUT / DELETE |
| `/special-days/for-week/{year}/{week}` | `get_special_days_for_week` (L694) | GET |
| `/report?year=&until_week=`, `/report/{id}?year=&until_week=`, `/report/week/{year}/{week}` | `get_short_reports` (L357), `get_employee_reports` (L374), `get_working_hours_for_week` (L542), `get_balance_until_week` (L556) | GET |
| `/permission/user[/]`, `/permission/role`, `/permission/user-role`, `/permission/user/{id}/roles` | `get_all_users` (L573), `get_all_roles` (L583), `get_roles_from_user` (L593), `add_role_to_user` (L610), `remove_role_from_user` (L619), `add_user` (L631), `delete_user` (L641) | GET / POST / DELETE |
| `/week-message`, `/week-message/{id}`, `/week-message/by-year-and-week/{year}/{week}` | `get_week_message` (L887), `post_week_message` (L907), `put_week_message` (L923) | GET / POST / PUT |
| `/billing-period[/{id}]`, `/billing-period/{id}/custom-report/{template_id}` | `get_billing_periods` (L965), `get_billing_period` (L975), `post_billing_period` (L988), `delete_billing_period` (L955), `generate_custom_report` (L1081) | GET / POST / DELETE |
| `/text-templates[/{id}]`, `/text-templates/by-type/{type}` | `get_text_templates` (L1003), `get_text_templates_by_type` (L1013), `get_text_template` (L1029), `create_text_template` (L1042), `update_text_template` (L1056), `delete_text_template` (L1071) | GET / POST / PUT / DELETE |
| `/block-report/{template_id}` | `generate_block_report` (L1099) | GET |
| `/blocks/{from_year}/{from_week}/{to_year}/{to_week}` | `get_blocks` (L1252) | GET |
| `/user-invitation/invitation[/{id}]`, `/user-invitation/invitation/user/{username}`, `/user-invitation/invitation/{id}/revoke-session` | `generate_invitation` (L1113), `list_user_invitations` (L1127), `revoke_invitation` (L1173), `revoke_session_for_invitation` (L1186) | GET / POST / DELETE |

There are 1269 lines in `src/api.rs`; every helper there is a backend call.

### API Contract: the Vendored `rest-types` Crate

**KNOWN CONSTRAINT — drifted fork.** The frontend depends on a *local copy* of
`rest-types` at `rest-types/` (path dependency declared at `Cargo.toml:28-29`).
This crate is a **drifted fork** of the backend's own `rest-types` crate.

- Frontend copy: `rest-types/src/lib.rs` — 1468 lines, `version = "1.0.5-dev"`.
- Backend copy: `../shifty-backend/rest-types/src/lib.rs` — 2041 lines, version
  `1.13.0-dev`.
- The two files **differ by ~816 lines** (`diff` byte-count). The drift is
  systematic, not accidental:
  - The frontend version drops every backend-only `From<&service::...>` impl
    and the `service-impl` feature-gate that powers them. Backend types behind
    the `service-impl` feature reach into `service::*` and `shifty_utils::*`
    crates that the frontend does not have.
  - The frontend struct definitions add `PartialEq, Eq` derives that the
    backend ones lack (e.g. `ShiftplanTO`, `rest-types/src/lib.rs:14`), to
    satisfy Dioxus reactivity and `GlobalSignal` equality checks.
  - The backend has fields the frontend has not yet adopted, e.g.
    `SlotTO::max_paid_employees` (Phase 5 D-10 marker), additional
    `ExtraHoursReportCategory::UnpaidLeave` and `VolunteerWork` variants,
    `ShiftplanSlotTO::current_paid_count`, `unavailable: Option<...>` on
    week views, `cap_planned_hours_to_expected`, `unpaid_leave_hours`,
    `volunteer_hours`, `absence_days` ordering changes, etc.
  - The frontend has at least one type the backend lacks at the same point:
    `ShiftplanDayAggregateTO`, `PlanDayViewTO` (frontend
    `rest-types/src/lib.rs:949-962`).
- Cargo manifests also diverge (`rest-types/Cargo.toml`):
  - Frontend `time = "0.3.41"` with full feature set;
  - Backend `time = "0.3.36"` with `serde-human-readable` only;
  - Backend declares `[lints.rust]` / `[lints.clippy]` gates as `deny`; the
    frontend copy does not.

**Impact:**
- Any backend payload that introduces a new `value_type`, a new field, or a
  new variant **silently round-trips with the missing field elided** as long
  as the backend uses `#[serde(default)]` / `skip_serializing_if`. Anything
  more invasive (renaming, retyping, removing the `$version` rename) breaks
  deserialisation in the frontend.
- New REST endpoints added to the backend require *manual* mirroring of any
  request/response struct into `shifty-dioxus/rest-types/src/lib.rs` plus the
  matching call helper in `src/api.rs`. There is **no codegen** — neither
  `openapi.json` (committed at the repo root, ~42 KB) nor `utoipa` is wired
  into the frontend build.
- Treat any task that touches the backend `rest-types` as a multi-crate change
  that must also patch the frontend's vendored copy. The drift means a naive
  copy-paste of the backend file will both break the frontend build (missing
  `service` deps) and regress the `PartialEq` derives that pages rely on.

## Authentication & Identity

**Mode:** OIDC session cookies, validated by the backend.

- The frontend has no client-side OIDC library. It calls `GET
  {backend}/auth-info` (`src/api.rs:22`) and trusts the backend's verdict:
  - HTTP 200 + JSON ⇒ authenticated; populates
    `AUTH: GlobalSignal<AuthStore>` (`src/service/auth.rs:50`).
  - Any non-200 ⇒ unauthenticated; `Auth` component (`src/auth.rs:15-23`)
    swaps in the `NotAuthenticated` page.
- `AuthInfo` schema: `src/state/auth_info.rs:6-11` —
  `{ user: Rc<str>, privileges: Rc<[Rc<str>]>, authenticated: bool }`.
  Privilege check helper: `AuthInfo::has_privilege(&str)` (line 24).
- Re-auth trigger: `src/error.rs:22-23` reloads the window when reqwest
  reports HTTP 401, kicking the browser back through the backend's redirect
  flow.
- OIDC silent-renew is bootstrapped at the HTML layer:
  - `<iframe id="silentRenewIframe">` at `index.html:29`.
  - 5-minute keep-alive `setInterval` that pings
    `window.oidcLoginKeepAliveURL` if the backend has set one
    (`index.html:31-39`).
  - `window.oidcLoginKeepAliveURL` is **never assigned by Rust code**; it is
    expected to be set by the backend's bootstrap response (e.g. an injected
    `<script>` or future server-rendered template). Today this means the keep-
    alive is effectively dormant unless something else writes the variable.
  - **Operator-confirmed (2026-05-07):** The dormant iframe + keep-alive code is
    not an active bug — production runs stable on the backend-cookie-session
    model (`/auth-info` ping + 401-triggered reload above). The iframe is
    likely a leftover from an earlier browser-side OIDC client setup. Origin
    and intended future use are open — see
    `.planning/todos/pending/2026-05-07-review-silent-renew-iframe-in-index-html.md`.
- There is no mock-auth toggle on the frontend; a development backend with the
  `mock_auth` feature is the standard way to log in without a live OIDC IdP.

## Dev Server Proxy

`Dioxus.toml` declares 25 reverse-proxy entries pointing at
`http://localhost:3000` (lines 45–92). When `dx serve` runs on port 8080 it
forwards the listed prefixes to the backend so that `assets/config.json` can
ship with `backend = "http://localhost:8080"` and avoid CORS / cookie-domain
issues during development.

Proxied prefixes (each `[[web.proxy]] backend = "http://localhost:3000/<p>"`):

```
/billing-period       /blocks                /block-report
/slot                 /auth-info             /booking
/custom-extra-hours   /special-days          /booking-information
/booking-log          /sales-person          /authenticate
/report               /extra-hours           /working-hours
/version              /permission            /shiftplan-edit
/shiftplan-info       /shiftplan-catalog     /week-message
/text-templates       /user-invitation       /sales-person-shiftplan
```

Note: `/authenticate` is in the proxy list (`Dioxus.toml:67-68`) but **no
caller in `src/api.rs` references it** — the frontend never POSTs login
credentials directly; the OIDC redirect handles authentication outside the
SPA. Keep the proxy entry to avoid breaking the OIDC redirect path, but do
not add new direct callers without coordinating with the backend auth flow.

Any new backend prefix must be added here as well, otherwise the dev build
will see `404` from the dev server's own port.

## Webhooks & Callbacks

**Incoming:** None. The frontend does not expose endpoints.

**Outgoing:** None beyond the REST calls listed above.

## Data Storage

- **Server-side:** all persistence flows through the backend REST API; no
  IndexedDB / WebSQL usage in Rust code.
- **Client-side:**
  - `localStorage["shifty-theme"]` — `"dark"` / `"light"` preference, read by
    inline JS at `index.html:13` and from Rust via `web_sys::Storage`
    (theme service `src/service/theme.rs`).
  - No sensitive data is persisted in `localStorage` / cookies by frontend
    code; the OIDC session cookie is set and managed entirely by the backend.

## Monitoring & Observability

- **Logging:** `tracing` + `dioxus-logger` to the browser console at
  `Level::INFO` (`src/main.rs:27`). The pattern in `src/api.rs` is
  `info!("Fetching ...")` / `info!("Fetched")` around each request.
- **Error tracking:** none — `src/error.rs` only `eprintln!`s.
- **Metrics / RUM:** none.

## CI/CD & Deployment

- **CI:** GitHub Actions directory exists at `.github/`, but no workflows are
  documented in this scan. (Out of scope for this `tech` focus; see project-
  level docs.)
- **Hosting:** static-file hosting of the `dist/` output. Production is
  packaged through Nix (`flake.nix` `frontend-build` derivation, lines
  33–174). The `shifty-nix` repository in the parent workspace is the
  deployment driver (referenced from the root `CLAUDE.md`).

## Required Runtime Configuration

The frontend hard-fails to render until `assets/config.json` is fetched and
`config.backend` is non-empty (`src/app.rs:28`). Required fields and their
defaults:

| Field | Required? | Default | Source |
|-------|-----------|---------|--------|
| `backend` | yes (non-empty) | — (empty `Rc<str>`) | `assets/config.json` |
| `application_title` | no | `"Shifty"` | `default_application_title` (`src/state/config.rs:9`) |
| `is_prod` | no | `false` | serde default |
| `env_short_description` | no | `"DEV"` | `default_env_short_description` (`src/state/config.rs:5`) |
| `show_vacation` | no | `false` | `default_show_vacation` (`src/state/config.rs:13`) |

There are **no environment variables** read by Rust at runtime (WASM has no
process env); all config flows through the JSON file at deploy time.

## External Asset Dependencies

- **Google Fonts** — Inter and JetBrains Mono families requested at
  `index.html:23-25` via `fonts.googleapis.com` / `fonts.gstatic.com`.
  Production deploys behind strict CSPs need to allowlist these origins or
  vendor the fonts.

---

*Integration audit: 2026-05-07*
