//! Phase 4 OpenAPI snapshot lock (D-Phase4-11).
//!
//! Wave 0: scaffold-only — the test is `#[ignore]`'d until Wave 2 has added the
//! `/admin/cutover/*` endpoints + `ExtraHoursCategoryDeprecatedErrorTO` schema.
//! Wave 2: removes the `#[ignore]`, runs `cargo test -p rest --test openapi_snapshot`
//! once to generate `rest/tests/snapshots/openapi_snapshot__openapi_snapshot_locks_full_api_surface.snap.new`,
//! a human reviews via `git diff` (no global `cargo insta` install per MEMORY.md),
//! then renames `.snap.new → .snap` (or runs `cargo insta accept` if locally installed).

use rest::ApiDoc;
use utoipa::OpenApi;

#[test]
#[ignore = "wave-2-accepts-snapshot"]
fn openapi_snapshot_locks_full_api_surface() {
    let openapi = ApiDoc::openapi();
    insta::with_settings!({ sort_maps => true }, {
        insta::assert_json_snapshot!(openapi);
    });
}
