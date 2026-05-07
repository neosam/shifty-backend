# Technology Stack

**Analysis Date:** 2026-05-07

This document describes the **frontend** crate (`shifty-dioxus`) at
`/home/neosam/programming/rust/projects/shifty/shifty-dioxus/`. All file paths
in this document are relative to that directory.

## Languages

**Primary:**
- Rust (edition 2021) — entire application code (`src/**/*.rs`); compiled to
  WebAssembly. Crate version `1.13.0-dev` (`Cargo.toml`).

**Secondary:**
- HTML (template) — `index.html` is the Dioxus shell with a small bootstrap
  script that imports the generated WASM bindings and a hidden
  `silentRenewIframe` plus an `oidcLoginKeepAliveURL` keep-alive ping.
- CSS — Tailwind-driven; source in `input.css`, generated bundle in
  `assets/tailwind.css`. Design tokens declared as CSS custom properties at
  `:root` (`input.css`).
- JavaScript (inline only) — theme bootstrap and OIDC keep-alive `setInterval`
  in `index.html`. No standalone JS files committed.

## Runtime

**Environment:**
- Browser WebAssembly target `wasm32-unknown-unknown`. Configured for the
  toolchain in `flake.nix` (line 22) and exported via
  `CARGO_BUILD_TARGET = "wasm32-unknown-unknown"` (line 64) and
  `RUST_TARGET = "wasm32-unknown-unknown"` (line 202) for the dev shell.
- Dev server is `dioxus-cli` (`dx serve`), see `flake.nix` shellHook
  (lines 205–217). Default platform `web` per `Dioxus.toml` line 8.

**Package Manager:**
- Cargo. Lockfile present (`Cargo.lock`, ~79 KB).
- Nix flake `flake.nix` / `flake.lock` for a reproducible dev shell and
  packaged build (`pkgs.rustPlatform.buildRustPackage`, line 33).
- No `npm`/`package.json` — Node tooling (`tailwindcss`, `nodejs`) is provided
  through the Nix dev shell (`flake.nix` lines 184–200).

## Frameworks

**Core:**
- `dioxus = "0.6.1"` with features `["web", "router"]` (`Cargo.toml` line 10) —
  the Rust/WASM UI framework. Application entry: `src/main.rs` invokes
  `launch(app::App)` from `src/app.rs`.
- `dioxus-logger = "0.6.2"` — logger init in `src/main.rs` line 27.
- `dioxus-router` (feature of `dioxus`) — route enum at `src/router.rs`
  (e.g. `Route::ShiftPlanDeep { year, week }`).
- `manganis = "0.6.2"` — asset pipeline; `asset!()` macro used at
  `src/app.rs:43` (Tailwind stylesheet) and image refs in
  `src/page/home.rs:33`, `src/page/not_authenticated.rs:19`.

**Testing:**
- `wasm-bindgen-test = "0.3"` (dev) — browser-targeted unit tests.
- `tokio-test = "0.4"` (dev) — async test helpers.
- `mockito = "1.2"` (dev) — HTTP mock server for API integration tests.
- `dioxus-ssr = "0.6"` (dev) — server-side rendering used in component tests
  under `src/tests/`.

**Build/Dev:**
- `dioxus-cli` (`dx`) — dev server with hot reload (`flake.nix` line 211 lists
  `dx serve`, `dx build`).
- `tailwindcss` — CSS pipeline. Watch script: `run-tailwind.sh` runs
  `tailwindcss -i ./input.css -o ./assets/tailwind.css --watch`. Production
  minify call inside `flake.nix` build phase line 77.
- `wasm-bindgen-cli` (pinned `wasm-bindgen-cli_0_2_104`) — generates JS
  bindings (`flake.nix` line 90).
- `binaryen` / `wasm-opt` — optimisation pass `-Oz` with debug-strip flags
  (`flake.nix` lines 100–113). Note: `DIOXUS_WASM_OPT_DISABLE=1` is set during
  build (line 73) because the Dioxus-CLI-driven `wasm-opt` failed; a manual
  `wasm-opt` pass replaces it.
- `lld` linker (`CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER = "lld"` in
  `flake.nix` line 203).
- `cargo-watch` available in dev shell (line 195).
- `wasm-pack`, `wasmtime`, `pkg-config`, `openssl` listed as dev shell tools.

## Key Dependencies

**Critical (runtime):**
- `reqwest = "0.12.15"` with `["json"]` — HTTP client used throughout
  `src/api.rs`. The WASM build implicitly uses the browser `fetch` backend.
- `serde = "1.0.219"` (`["rc"]`) and `serde_json = "1.0.140"` — DTO
  serialisation. The `rc` feature is required because most `*TO` types use
  `Arc<str>` / `Rc<[T]>`.
- `uuid = "1.17"` (`["v4", "js"]`) — IDs in REST payloads. The `js` feature
  pulls a browser-safe randomness source.
- `time = "0.3.41"` with `["macros", "serde-human-readable", "parsing",
  "formatting", "serde", "std"]` — date/time domain types; bridged to JS
  `Date` via helpers in `src/js.rs`.
- `web-sys = "0.3.77"` with a wide feature set incl. `Window`, `Storage`,
  `MediaQueryList`, `KeyboardEvent`, `HtmlTextAreaElement`, `DomRect`,
  etc. (`Cargo.toml` lines 44–62) — used for theme bootstrap, clipboard
  fallback (`src/js.rs`), media-query reactivity
  (`src/component/atoms/media_query.rs`), and document title updates
  (`src/app.rs:32`).
- `wasm-bindgen = "0.2.97"` and `wasm-bindgen-futures = "0.4.47"` — JS interop.
  See `extern "C"` clipboard binding at `src/js.rs:40-49`.
- `js-sys = "0.3.77"` — `js_sys::Date`, `js_sys::Function`, `Reflect` for the
  `execCommand` clipboard fallback in `src/js.rs:74-127`.
- `gloo-timers = "0.3.0"` (`["futures"]`) — debouncing / delayed coroutines.
- `futures = "0.3.30"`, `futures-util = "0.3.30"` — `StreamExt` for service
  coroutine receivers (`src/service/config.rs:35`).
- `async-recursion = "1.1.1"` — recursive async helpers in service layer.
- `thiserror = "2.0.17"` — `ShiftyError` enum at `src/error.rs:5`.
- `tracing = "0.1.41"` — application logging; level `INFO` at startup
  (`src/main.rs:27`).

**Local path dependency:**
- `rest-types = { path = "rest-types" }` — the **vendored** API contract crate.
  See INTEGRATIONS.md for the drift caveat.

## Configuration

**Build / framework:**
- `Dioxus.toml` — application name `shifty-dioxus`, `default_platform = "web"`,
  `out_dir = "dist"`, `asset_dir = "assets"`, watcher monitors `["src",
  "assets"]`, served stylesheet `/tailwind.css`, plus 25 `[[web.proxy]]`
  entries pointing at `http://localhost:3000` (lines 45–92).
- `tailwind.config.js` — content globs `./src/**/*.{rs,html,css}` and
  `./dist/**/*.html`; design-token aliases bound to CSS variables; explicit
  `safelist` for dynamically constructed classes (lines 68–84).
- `input.css` — Tailwind directives plus design-token CSS variables and the
  body baseline typography (sourced from `shifty-design/project`).
- `flake.nix` — Nix package + dev shell; pins toolchain to
  `pkgs.rust-bin.stable.latest.default` with `wasm32-unknown-unknown` target
  and `rust-analyzer`. Build phase performs Tailwind compile, raw cargo wasm
  build, manual `wasm-bindgen` + `wasm-opt` passes, and emits a self-contained
  static site to `dist/`.
- `default.nix` — present (3.4 KB), legacy entry alongside the flake.

**Runtime configuration:**
- `assets/config.json` — fetched by `src/api.rs:34-53` (`load_config`) at
  startup. Schema in `src/state/config.rs:18-28` (`Config { backend,
  application_title, is_prod, env_short_description, show_vacation }`). The
  `backend` field is the base URL the entire `src/api.rs` formats requests
  against. Committed config points at `http://localhost:8080`.
- `index.html` — theme bootstrap reads `localStorage["shifty-theme"]` and
  applies `data-theme="dark|light"`. Loads Inter / JetBrains Mono from Google
  Fonts (lines 23–25). OIDC silent-renew iframe + 5-minute keep-alive
  `fetch(oidcLoginKeepAliveURL)` at lines 29–39.
- `assets/manifest.json` referenced by `index.html` line 6 — file is **not
  committed** to the repo (only `tailwind.css` lives under `public/`). It
  must be supplied at deploy time.

**Profiles** (`Cargo.toml` lines 64–74):
- `wasm-dev` inherits `dev`, `opt-level = 1`.
- `server-dev`, `android-dev` inherit `dev` (placeholders; no server / mobile
  build is wired up).

## Platform Requirements

**Development:**
- NixOS / nix-with-flakes (project standard). Enter dev shell via
  `nix develop` (per local memory `reference_local_dev_commands.md`).
- Rust toolchain `stable.latest` with `wasm32-unknown-unknown` target,
  `rust-src`, `rust-analyzer`.
- Two-process workflow:
  - Terminal 1: `npx tailwindcss -i ./input.css -o ./assets/tailwind.css
    --watch` (or `./run-tailwind.sh`).
  - Terminal 2: `dx serve --hot-reload` on port 8080 (proxies REST calls to
    the backend on `http://localhost:3000`).
- Dev shell tools: `rustToolchain`, `wasm-pack`, `wasm-bindgen-cli_0_2_104`,
  `wasmtime`, `dioxus-cli`, `nodejs`, `nodePackages.npm`, `tailwindcss`,
  `pkg-config`, `openssl`, `cargo-watch`, `lld`, `binaryen`, `openspec`, `gsd`.

**Production:**
- Static site: the Nix build produces `dist/` containing optimised
  `shifty-dioxus.wasm` + JS bindings + `tailwind.css` + a minimal `index.html`
  (`flake.nix` lines 117–139). The output is purely static — any web server
  capable of serving the files plus configuring proxy/CORS to the backend
  will do. Note the production `index.html` baked by `flake.nix` (lines
  117–134) is **simplified** and does not include the OIDC silent-renew iframe
  / keep-alive ping that the dev `index.html` ships — callers relying on those
  in production must regenerate `index.html` from the dev template or extend
  the Nix build phase.
- WASM size optimisation: `wasm-opt -Oz` with `--strip-debug --strip-dwarf
  --strip-producers`, plus `nix`-managed `remove-references-to` to strip Nix
  store paths from the binary (`flake.nix` lines 100–114, 155–156).

## Build Flow Summary

```
input.css ──► tailwindcss -i input.css -o assets/tailwind.css [--watch|--minify]
                                  │
src/**/*.rs ─► cargo build --target wasm32-unknown-unknown --release
                                  │
                       target/.../shifty-dioxus.wasm
                                  │
                       wasm-bindgen --out-dir dist --target web
                                  │
                       wasm-opt -Oz --strip-debug ...
                                  │
                       remove-references-to (Nix only)
                                  │
                       dist/{shifty-dioxus.wasm, shifty-dioxus.js,
                             tailwind.css, index.html, manifest.json}
```

In dev (`dx serve`) the same chain is orchestrated by the Dioxus CLI with hot
reload of `src/` and `assets/`.

---

*Stack analysis: 2026-05-07*
