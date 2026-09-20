# Deferred Items — Phase 63

## Aus Plan 63-01

- **Pre-existing clippy-Warnung unter `--all-targets`** —
  `service_impl/src/test/person_directory_dao.rs:149` (`needless_lifetimes`
  auf `fn row_of<'rows>`). Stammt aus Phase 62, von 63-01 nicht beruehrt.
  Der in CLAUDE.md festgelegte Gate `cargo clippy --workspace -- -D warnings`
  ist gruen; nur die erweiterte Variante `--all-targets` schlaegt an.
  Ausserhalb des Scopes von 63-01.

## Aus Plan 63-02 (2026-09-20)

- `service_impl/src/test/person_directory_dao.rs:149` — `clippy::needless_lifetimes`
  auf `fn row_of<'rows>(...)`. Vorbestehend aus Phase 62, feuert nur unter
  `cargo clippy --workspace --all-targets`, NICHT unter dem CLAUDE.md-Gate
  (`cargo clippy --workspace -- -D warnings`) und nicht in CI. Ausserhalb des
  Scopes von 63-02 (fremde Datei, nicht von diesem Plan beruehrt).
- `cargo fmt` ist in der Nix-Dev-Shell nicht verfuegbar (`no such command: fmt`),
  obwohl CI ein fmt-Gate faehrt. Betrifft die Toolchain, nicht diesen Plan.
