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

## Aus Plan 63-03 (2026-09-20)

- **Vorbestehende Doppelregistrierung von `Key::CopyToClipboard`** —
  `shifty-dioxus/src/i18n/de.rs:579` + `:645` und
  `shifty-dioxus/src/i18n/en.rs:532` + `:592`. Der zweite `add_text`-Aufruf
  ueberschreibt den ersten still ("Kopieren" -> "In die Zwischenablage kopieren"
  bzw. "Copy" -> "Copy to Clipboard"); `cs.rs` hat nur einen Eintrag, die drei
  Sprachen sind damit inhaltlich uneinheitlich. Stammt aus einer frueheren
  Phase, von 63-03 nicht beruehrt (T-63-03-C betrifft nur die 32 neuen Keys).
  Ausserhalb des Scopes von 63-03 — eigener Aufraeum-Pass noetig, inkl. der
  Entscheidung, welcher der beiden Texte gewinnen soll.
- `cargo fmt` weiterhin nicht in der Nix-Dev-Shell verfuegbar (`no such
  command: fmt`). Die neuen i18n-Bloecke wurden deshalb per Hand auf die
  rustfmt-Zeilenbreite (100) gebracht — lange Strings in der mehrzeiligen
  `add_text`-Form, kurze einzeilig. Betrifft die Toolchain, nicht diesen Plan.
