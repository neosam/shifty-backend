# Deferred Items — Phase 39

## Out-of-scope pre-existing test failures (found during 39-04)

- **`i18n::tests::i18n_impersonation_keys_match_german_reference`** fails on the
  current tree independent of this plan's changes. The de.rs value for
  `Key::ImpersonateActAs` is `"🥸 Agieren"` (committed in `83a0d91`, feat 37-02),
  but the reference test still expects `"Als diese Person agieren"`. Unrelated to
  the KW-Status work — the reference assertion or the label needs reconciling in a
  dedicated fix. Not touched by 39-04 (Scope Boundary rule).
