# Deferred Items — Phase 55

## 55-03 (Out-of-scope, pre-existing)

- **`cargo clippy --workspace --all-targets` trips `clippy::cloned-ref-to-slice-refs`**
  in `service_impl/src/test/voluntary_stats.rs` (Zeilen 363, 403, 944, 945:
  `&[abs.clone()]` → `std::slice::from_ref(&abs)`).
  - Pre-existing (Datei zuletzt in `e2aee9b6`, quick-260710-p8k — nicht von 55-03
    beruehrt). Neuere Clippy-Version meldet den Lint nur unter `--all-targets`.
  - Der dokumentierte Projekt-Gate `cargo clippy --workspace -- -D warnings`
    (lib/bin-Targets, CLAUDE.md) ist gruen. SCOPE BOUNDARY: nicht durch 55-03
    verursacht → nicht gefixt.
  - Fix (trivial): die vier `&[abs.clone()]`-Aufrufe auf
    `std::slice::from_ref(&abs)` umstellen.
