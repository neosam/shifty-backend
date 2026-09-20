# Deferred Items — Phase 63

## Aus Plan 63-01

- **Pre-existing clippy-Warnung unter `--all-targets`** —
  `service_impl/src/test/person_directory_dao.rs:149` (`needless_lifetimes`
  auf `fn row_of<'rows>`). Stammt aus Phase 62, von 63-01 nicht beruehrt.
  Der in CLAUDE.md festgelegte Gate `cargo clippy --workspace -- -D warnings`
  ist gruen; nur die erweiterte Variante `--all-targets` schlaegt an.
  Ausserhalb des Scopes von 63-01.
