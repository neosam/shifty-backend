# Codebase Concerns

**Analysis Date:** 2026-05-05

## Tech Debt

**Serde Deserialization Error Conversion:**
- Issue: In `src/api.rs:1167`, TODO comment notes poor error handling when converting serde deserialization failures to reqwest errors
- Files: `src/api.rs` (line 1167)
- Impact: When invitation data fails to deserialize, the function logs the error but returns an empty array `Ok(Rc::new([]))` instead of properly propagating the error. This masks failures and makes debugging harder
- Fix approach: Define a custom error type that wraps both serde and reqwest errors, or use `anyhow::Result` with proper context preservation

**Excessive Unwrap/Expect Calls:**
- Issue: 462 instances of `unwrap()`, `expect()`, and `panic!` throughout the codebase
- Files: Concentrated in:
  - `src/api.rs` (lines 37, 40, 42 — window/location access)
  - `src/error.rs` (line 23 — window reload)
  - `src/app.rs` (lines 33-34 — DOM access)
  - `src/component/week_view.rs` (lines 155, 193, 198-200, 760, 1359-1360, 1382 — critical component)
- Impact: Runtime panics possible in edge cases (missing window, network failures, DOM access failures). The WeekView component (1,631 lines) is especially fragile
- Fix approach: Use `Result<T, E>` or `Option<T>` returns instead. Provide graceful fallbacks for web_sys access

**Clone-Heavy Data Flow:**
- Issue: 617 uses of `.clone()`, `.cloned()`, and vector allocations throughout the codebase, especially in loader and service layers
- Files: `src/loader.rs` (collection clones during transformation), `src/page/shiftplan.rs` (page state clones)
- Impact: Unnecessary memory allocations during data transformations, potential performance degradation with large datasets
- Fix approach: Use references where possible, adopt `Copy` semantics for small types, reduce intermediate Vec allocations

## Known Bugs & Recent Fragility

**WeekView Slot Rendering Complexity (Recent Fix):**
- Issue: Slots with overflow had complex height calculations that required multiple fixes
- Files: `src/component/week_view.rs` (1,631 lines)
- History: 
  - Commit `6b9d916` (Apr 28): Fixed slot overflow with fixed-height cell and internal chip scroll
  - Commit `f7ecca1` (Apr 28): Fixed vertical scrollbar caused by fractional hour boundaries (day_start not snapped to whole hours)
- Why fragile: Complex absolute positioning with dynamic height calculation based on time ranges. WeekView combines CSS zoom, absolute positioning, and scrolling logic in a single 1,631-line component
- Safe modification: Any changes to height calculation, scrolling, or position logic must be tested with slots that have fractional boundaries (e.g., 19:30 end times)
- Test coverage: `src/component/week_view.rs` has inline tests (lines 1348-1630) covering chip rendering and slot offset snapping

**Zoom Property Rendering (Historical Issue):**
- Issue: WeekView must use CSS `zoom` property, NOT `transform: scale`
- Files: `src/component/week_view.rs` (scaling logic)
- Why it was fragile: Previous attempts used `transform: scale` which created gap artifacts
- Current status: Fixed; zoom is now CSS-based (SCALING constant = 75.0)
- Risk: Developers unfamiliar with this constraint may revert to scale transforms

**German Translation Locale Bug (Historical Issue):**
- Issue: German translations previously used `Locale::En` instead of `Locale::De`
- Files: `src/i18n/` modules
- Status: Fixed in `cfa7911` (commit message: "fix: i18n german language key")
- Risk: New translation additions could repeat this pattern if not reviewed carefully
- Safe modification: Always verify that translation getters use correct locale enums

**Dark Mode Page Background (Recent Fix):**
- Issue: Page background did not switch to dark theme
- Files: `input.css`, `src/` (multiple pages)
- Commit: `f5c7483` (Apr 27): Added missing html/body background rule from reference design
- Why fragile: CSS reset rules can be overridden by page-level styles; missing root element background rules
- Risk: New page components may not inherit dark theme properly if background tokens are not applied to root elements

## Security Considerations

**Session/Auth Token Storage:**
- Risk: Frontend doesn't explicitly control token storage; relies on backend session management
- Files: `src/service/auth.rs`, `src/api.rs` (fetch_auth_info at line 22-31)
- Current mitigation: Auth info fetched on app init via `fetch_auth_info()` call to `/auth-info` endpoint. No credentials stored in localStorage detected
- Recommendations:
  - Document auth session model (cookie-based vs token-based)
  - Ensure auth tokens use HttpOnly cookies if session-based
  - Add CSRF protection headers if using state-changing requests

**XSS Risk via RSX:**
- Risk: Dioxus RSX prevents direct HTML injection, but user-provided data rendered in slots could be vulnerable if sanitization is missing
- Files: Components rendering user-supplied text (booking names, employee names)
- Current mitigation: Rust type system + RSX prevents most XSS, but string data from API should be validated
- Recommendations: Audit API response handling for user-controlled fields (names, labels, descriptions)

**Unauthorized 401 Handling:**
- Risk: Hardcoded full page reload on 401 response could lose unsaved state
- Files: `src/error.rs` (line 23)
- Current behavior: Any 401 triggers `window.location.reload()`
- Impact: User form data, pending operations lost without warning
- Fix approach: Show modal confirmation before reload, offer to save pending changes, log out cleanly

**URL Invitation Token Exposure:**
- Risk: Invitation tokens exposed in URL path (loader.rs:813)
- Files: `src/loader.rs` (line 813: `format!("{}/auth/invitation/{}", backend_url, invitation.token)`)
- Current mitigation: HTTPS (assumed) encrypts in transit, but tokens in URLs can leak via:
  - Browser history
  - Referrer headers
  - Log files
- Fix approach: Use POST/callback mechanism instead of token-in-URL, or add URL token expiry with server-side validation

## Performance Bottlenecks

**Large Component Render Cycles:**
- Problem: WeekView (1,631 lines), ShiftPlan (1,377 lines), TopBar (1,166 lines), and EmployeeView (997 lines) are monolithic
- Files: `src/component/week_view.rs`, `src/page/shiftplan.rs`, `src/component/top_bar.rs`, `src/component/employee_view.rs`
- Cause: These components manage too much state and re-render too frequently. For example, WeekView handles:
  - Day layout, time column rendering, slot positioning
  - Tooltip triggers and positioning
  - Dropdown menus
  - Scrolling state
- Improvement path:
  - Split WeekView into sub-components (TimeColumn, DayColumn, SlotArea)
  - Use `use_memo` to cache layout calculations
  - Separate tooltip/dropdown logic into child components with independent lifetimes

**Excessive Collection Cloning During Data Load:**
- Problem: `src/loader.rs` clones data multiple times during transformation
- Files: `src/loader.rs` (load_bookings function, line 76+)
- Cause: Data flows `API → TO structs → State structs → Rc<[State]>` with multiple `.map()` and `.collect()` calls
- Example: `sales_person_tos.iter().map(SalesPerson::from).collect()` allocates intermediate Vec, then converts to Rc
- Improvement path: Use iterators directly where possible, avoid intermediate collections

**API Error Deserialization Penalty:**
- Problem: Failed JSON deserialization in `src/api.rs:1160-1171` falls back to logging + returning empty array
- Files: `src/api.rs` (fetch_invitations function)
- Impact: No user feedback, silent failure makes debugging slow
- Improvement path: Implement proper error variant in API result type

## Fragile Areas

**WeekView Component (1,631 lines):**
- Files: `src/component/week_view.rs`
- Why fragile:
  - Complex CSS calculations mixing `zoom`, absolute positioning, and dynamic sizing
  - Timezone/daylight savings handling in time calculations
  - Tooltip positioning calculations based on window dimensions (lines 198-200 unwraps)
  - Multiple recent bug fixes (April 28) indicate instability
- Safe modification:
  - Extract time calculation logic to separate utility module
  - Add extensive test coverage for edge cases (fractional hours, midnight boundaries, empty days)
  - Use computed properties for position/size calculations instead of inline style strings
- Test coverage: Inline tests present (lines 1348-1630) but limited to chip count and offset snapping

**ShiftPlan Page (1,377 lines):**
- Files: `src/page/shiftplan.rs`
- Why fragile:
  - Multiple service handles for state management (WeeklySummaryAction, BookingConflictAction, etc.)
  - Complex event handling for add/remove bookings, copy from previous week, availability toggles
  - Mixed concerns: data loading, UI state, service orchestration
- Safe modification:
  - Document action flow before changes
  - Run full test suite (`cargo test`) after modifications
  - Check for state consistency issues in service handlers

**TopBar Component (1,166 lines):**
- Files: `src/component/top_bar.rs`
- Why fragile:
  - Routing state accessed directly (AUTH, router navigation)
  - Conditional rendering based on multiple auth roles
  - Recent change in `56099b2` grouped admin items into dropdown, which could cause menu nesting issues
- Safe modification:
  - Test all auth role combinations
  - Verify dropdown state resets on page navigation
  - Check mobile responsiveness after layout changes

**Dropdown/Menu System:**
- Files: Multiple (week_view, top_bar, etc. use dropdown_base)
- Why fragile:
  - Dropdown positioning calculated at render time (window.inner_width/height in week_view line 199-200)
  - Could break with very small viewports or hidden window objects
  - Recent dark-mode fixes touched styling (38257df) which may affect dropdown visibility
- Safe modification:
  - Test dropdown rendering on 320px width (mobile)
  - Verify positioning doesn't escape viewport on all screen sizes

## Scaling Limits

**Frontend Booking Limit:**
- Current capacity: UI renders all bookings for a week in WeekView grid
- Limit: ~200+ bookings per week causes noticeable render lag (no pagination implemented)
- Scaling path:
  - Implement virtual scrolling or pagination
  - Move heavy calculations to backend (pre-aggregated views)
  - Use Web Workers for data transformation

**API Response Size:**
- Current capacity: Fetches all slots, bookings, and sales persons for a single week
- Limit: No discovered limits, but error handling is silent (returns empty array on deserialization failure)
- Scaling path:
  - Add pagination/filtering query parameters to API
  - Lazy-load bookings on demand (scroll into view)
  - Implement caching with stale-while-revalidate pattern

**Global State Stores:**
- Concern: Multiple global signals (AUTH, I18N, CONFIG, BOOKING_LOG_STORE, etc.)
- Impact: App re-renders on any store update; no fine-grained reactivity
- Scaling path: Migrate to more granular state management (per-component signals instead of global)

## Dependencies at Risk

**Dioxus Framework:**
- Risk: Active development; breaking changes possible in minor releases
- Impact: Cargo.toml version constraints important for stability
- Current status: No version lock detected in grep (need to check Cargo.toml)
- Migration plan: Maintain version pinning, test before updating

**Reqwest HTTP Client:**
- Risk: Uses `.unwrap()` on window/location operations heavily
- Current usage: `src/api.rs` (line 36-44) unwraps web_sys::window multiple times
- Migration plan: Wrap web_sys calls in custom error type with graceful fallbacks

## Missing Critical Features

**Error Boundary / Graceful Degradation:**
- What's missing: No error boundary component to catch panics and show user-friendly error messages
- Blocks:
  - Users get blank page on panic instead of helpful message
  - No recovery path for transient failures
- Recommendation: Implement error boundary wrapping main app component

**Offline Support:**
- What's missing: No offline detection or service worker caching
- Blocks: Any network disruption causes silent failures
- Recommendation: Add offline detection, queue failed requests, show user messaging

**Request Cancellation:**
- What's missing: No way to cancel in-flight requests when navigating away
- Impact: Completed requests after navigation can update stale state, cause UI artifacts
- Recommendation: Implement abort signal pattern for fetch requests

## Test Coverage Gaps

**WeekView Edge Cases:**
- What's not tested: Slots with duration < 1 hour, overlapping bookings, all-day events, DST boundaries
- Files: `src/component/week_view.rs` (tests at lines 1348-1630)
- Risk: Recent fixes (April 28) suggest fragility around time calculations
- Priority: High — WeekView is most complex component

**Auth Flow:**
- What's not tested: 401 response handling, auth token expiry, re-authentication flow
- Files: `src/service/auth.rs`, `src/auth.rs`
- Risk: Error handler reloads page without user warning (line 23 in error.rs)
- Priority: High — Security-sensitive

**Mobile Responsiveness:**
- What's not tested: Dropdown positioning on mobile, WeekView zoom on small screens, touch event handling
- Files: All components with dropdown_base, week_view (zoom logic)
- Risk: Dropdowns could escape viewport, zoom could be inaccessible on mobile
- Priority: Medium — Affects user experience

**i18n Translation Completeness:**
- What's not tested: All UI text rendered with i18n keys, no hardcoded strings
- Files: `src/i18n/` and all component/page files
- Risk: Missing translations could show as "??" placeholders (unwrap_or_else in i18n.rs line 91)
- Priority: Medium — Could block international deployment

**Error Recovery:**
- What's not tested: Network errors, API timeouts, concurrent request races
- Files: `src/api.rs`, `src/loader.rs`, `src/service/`
- Risk: Silent failures when deserialization fails (api.rs line 1167)
- Priority: High — Affects reliability

---

*Concerns audit: 2026-05-05*
