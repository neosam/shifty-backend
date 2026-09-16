# Feature Research

**Domain:** Merged person/identity administration screen for an internal admin tool (shift-planning + HR, used by a handful of shop staff: shiftplanners, HR, volunteers)
**Researched:** 2026-09-16
**Confidence:** MEDIUM (no `gsd-tools`/Node runtime available in this environment — `node` is not on PATH, so the `research-plan`/`classify-confidence` seam could not run; confidence tiers below are self-assessed using the same hierarchy logic: official vendor docs > OSS project docs/issues > synthesis. Current-app claims are HIGH confidence — verified by reading the cited files directly.)

**Locked decisions this research builds on (from `.planning/seeds/personenverwaltung-zusammenlegen.md`, not reopened):** one entry concept spanning three shapes (User+SalesPerson / User-only / SalesPerson-only), one component that blends sections in/out, `Person` as a thin identity aggregate, `sales_person.name` and `user.name` stay put, list under `/user_management/` loses its tabs, "add login" / "add person" are actions on an entry, no snapshot bump.

## Current State (verified by reading the code)

- `/user_management/` already merged the **list** into one page, but not into one list: it is a tab bar (`UserMgmtTab::SalesPersons` default / `UserMgmtTab::Users`) with two independently-loaded, independently-searched tables (`shifty-dioxus/src/page/user_management.rs:39-115`, tab content at `:213-351` and `:353-496`). Each row already carries a cross-reference column today — the SalesPerson table has a "Linked User" column reading `sales_person_user_links` (`:265`, `:289-293`), the Users table has a "Linked Sales Person" column reading `user_sales_person_links` (`:403`, `:415-419`) — so the *data* for cross-linking is already wired into the store; only the **presentation** (two tables) and the **entry point** (detail pages) are split.
- Unlinked state today renders as plain muted text `i18n.t(Key::Unlinked)` (`shifty-dioxus/src/page/user_management.rs:325`, `:437`, `:443`, `:454`) — no badge, icon, or distinguishing color, on both sides symmetrically. There is no dedicated visual treatment for "sales person with no login" vs "user with no sales person" — both currently look identical (grey "—"/"Unlinked" label).
- Linking today happens by **typing a raw ID into a text field**, not by picking from a list: `sales_person_details.rs:189-222` renders a `TextInput` for `user_id` and calls `UpdateSalesPersonUser` with whatever string was typed, or an "unstyled" `ConnectUserAccount` button that seeds an empty string into the same field. There is no search/autocomplete/picker component in the current linking flow, and no confirmation before unlinking (`RemoveSalesPersonUser` fires immediately on trash-icon click, `:192-198`).
- Deleting a `user` (`DeleteUserConfirmDialog`, `user_management.rs:174-211`) asks for confirmation but the dialog text only ever mentions the username — it says nothing about a linked SalesPerson, and the action fires `DeleteUser` with no visible warning that a link would be severed.
- Three genuinely separate routes/pages exist per real person today: `/sales_person_details/:id`, `/user_details/:name`, `/employees/:id` (confirmed via `sales_person_details.rs`, `user_details.rs`, and named in the seed) — each with its own back button, its own header, its own loading state. `user_details.rs` shows Role Assignments + Invitations (Admin-gated territory); `sales_person_details.rs` shows Basic Info + Settings + Shiftplan Assignments (shiftplanner-gated territory) plus the one raw-text-field user link. Merging these correctly means each *section*, not the *page*, carries its own permission gate — this is explicitly still an open question in the seed (`personenverwaltung-zusammenlegen.md:174-177`).

## Feature Landscape

### Table Stakes (Users Expect These)

Features an admin managing a mixed roster of employees, HR accounts, and volunteers will assume exist. Missing these makes the merged screen feel like a regression from the current two-tab list, not an improvement.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Single list, one row per entry, with a **shape indicator** (badge/column) showing "Login only" / "Roster only" / "Login + Roster" | This is the entire point of the merge per the seed — users called the current split "extrem verwirrend". Precedent: Salesforce Person Accounts render one record type but the UI still shows which "layout" (business vs person) applies via a field/label, never hides that the shape differs. | LOW | Backend already has the join data (`sales_person_user_links`/`user_sales_person_links`); this is mostly a rendering + query-shape change, one table instead of two. |
| Search across the merged list by whichever name is present (username, sales-person name, or both) | Existing search already works per-tab (`matches_search`, `user_management.rs:32-37`); merging lists must not regress the ability to find someone by either name. Fat-Backend principle (`PROJECT.md:56-67`) says the merged display name should be computed server-side, not client-guessed. | LOW | Straightforward carry-over; the only new work is the fallback-name rule already flagged as open in the seed. |
| Filter by shape (e.g. "show only roster-only / login-only / linked") | GitLab's members list supports exactly this shape of filter for "invited but no account yet" vs active members; it is the standard way admin tools cope with heterogeneous membership states once entries stop being tab-separated. | LOW | Pure client-side filter over already-loaded data, or a query param — either is cheap for a "handful of people" dataset size. |
| "Add login" action on a roster-only entry, and "Add roster entry" action on a login-only entry, both **on the row/detail page**, not a separate top-level "create" flow | Locked by the seed already (`personenverwaltung-zusammenlegen.md:74-76`) — flagged here because it is genuinely table stakes, not a differentiator: it's the mechanism that makes "shapes are properties of one entry" true instead of aspirational. | MEDIUM | Needs both a "link to an *existing* login/roster row" path and a "create a *new* one" path — see next row. |
| Distinguish **"link an existing X"** from **"create a new X"** as two explicit choices, not one ambiguous text field | Current app conflates these badly: `ConnectUserAccount` sends an empty string into a raw `TextInput`, so the admin is expected to *type* an exact existing username with zero validation feedback until save. Keycloak's admin-UI backlog explicitly treats "invite existing user" and "invite new user" as two distinct, clearly-labelled actions/dropdown entries precisely because conflating them causes mistaken invites. | MEDIUM | Needs, at minimum, a picker/autocomplete over existing unlinked users (or unlinked sales-persons) plus a separate "create new" affordance — this is the single biggest UX upgrade over today's raw-text-field flow. |
| Confirmation before **unlinking** a login/roster pair, stating what disappears (roles/shiftplan visibility) not just "unlink?" | Today unlinking a SalesPerson's user is a single unconfirmed trash-icon click (`sales_person_details.rs:192-198`); deleting a user *does* confirm but doesn't mention the link (`user_management.rs:174-211`). Both are gaps relative to what any admin tool with destructive-ish actions on shared records should do — Auth0's account-linking guidance explicitly calls out that a "connect/disconnect account" UI must clearly state what happens to reduce accidental disconnects. | LOW | This is a copy/interaction-pattern fix on an existing dialog, not new plumbing — cheap relative to its safety value. |
| Section-level (not page-level) role gating on the merged detail view | Locked as an open question in the seed (`:174-177`) but it is table stakes, not optional — without it, either shiftplanners see Role/Invitation admin controls they shouldn't, or HR loses the ability to manage roster data for a login-only account they administer. | MEDIUM | This is the crux backend/frontend contract question for the phase: the DTO must tell the FE which sections are visible/editable per caller, per "Fat Backend, Thin Client" (`PROJECT.md:56-67`) — same pattern already used for HR-gated Nullable fields in v2.6 (voluntary-hours stats). |
| Deleting either half of a linked pair must have an explicit, distinguishable warning from deleting an unlinked entry | The current `DeleteUserConfirmDialog` text is identical regardless of whether the user has a linked SalesPerson or not — an admin can delete a person's login without realizing their shiftplan history/roster entry is untouched, or (worse, if a future "delete sales person" gets a symmetrical dialog) accidentally sever billing-relevant history. `sales_person.id` is referenced by twelve tables including write-once `billing_period_sales_person` snapshots (seed, `:87-95`) — deleting the SalesPerson side of a pair is categorically more dangerous than deleting the User side. | LOW–MEDIUM | Mostly a copy/branching change in existing confirm dialogs: "this will only remove the login" vs "this will remove roster history used in past billing periods" are different sentences, driven by which half is being deleted. |

### Differentiators (Nice, But Not Required for a Shop This Size)

Patterns seen in larger identity-admin products that would be over-engineering here — listed so the roadmap doesn't accidentally scope them in.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Automatic "suggested link" detection (e.g., matching a new user signup to an existing sales-person by name/email similarity) | Useful at the scale of thousands of accounts where manual matching doesn't scale (this is Auth0's "suggested account linking" pattern — matching by verified email). | MEDIUM–HIGH | A handful of staff means every merge candidate is obvious to a human; automatic fuzzy-matching adds false-positive risk (wrongly suggesting two different Lenas are the same person) for a problem that doesn't exist at this scale. Skip. |
| Full merge/split history & audit trail UI (who linked/unlinked whom, when, reversible) | Enterprise identity/MDM tools (Salesforce, PingIDM) treat this as core because compliance and multi-admin environments need it. | HIGH | One or two admins doing this occasionally do not need a dedicated audit UI beyond whatever general audit logging the app already has. Standard `updated_at`/existing audit columns on `person` suffice; a dedicated "history of link changes" screen is differentiator-tier, not needed now. |
| Bulk-link / bulk-import wizard (CSV of username↔sales-person pairs) | Relevant for onboarding a large org at once or migrating from another system. | MEDIUM | Not relevant here — the merge is a one-time backfill migration (already scoped as data-model-first in the seed), and day-to-day linking is one person at a time. |
| Cross-field "profile completeness" indicators / gamified onboarding nudges ("70% complete — add a phone number") | Common in consumer-facing profile UIs (LinkedIn-style). | LOW–MEDIUM | Wrong register for an internal admin tool; `person` intentionally carries no profile fields yet (seed: "Person ist das Aggregat, nicht der Datenträger", `:124-128`). Skip until/unless `person` gains real fields. |
| Multi-account linking (one login associated with several roster entries, or vice versa) | Relevant to systems modeling franchises/multi-tenant identity (PingIDM's "link multiple accounts to one identity"). | HIGH | Explicitly excluded by the domain: the old `sales_person_user` table enforced 1:1 via `UNIQUE(sales_person_id)` **and** `UNIQUE(user_id)` (seed, `:41-43`), and Variant A keeps that shape. A shop has exactly one login per person, one roster slot per person — no reason to relax this. |

### Anti-Features (Commonly Requested, Often Problematic)

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|------------------|-------------|
| Hard-delete a `person`/cascade-delete both halves in one click ("delete this person entirely") | Feels like the natural symmetric counterpart to "one entry, one delete button" | Deleting the SalesPerson half touches twelve referencing tables including write-once billing snapshots (seed, `:87-95`); a single "delete everything" button invites exactly the kind of accidental data loss the seed's "Unantastbar" section warns about. Also collides with existing soft-delete conventions (`WHERE deleted IS NULL`, per root `CLAUDE.md`). | Keep delete actions scoped per aggregate (delete-the-login vs deactivate-the-roster-entry), each with its own honest confirmation copy (see table-stakes row above); "inactive" already exists as the non-destructive path for SalesPerson (`sales_person_details.rs:170-187`). |
| A single shared "name" field on `Person` that both objects render from | Looks like it would simplify the merged list's header/label logic | Explicitly rejected by the locked decision "Namen bleiben, wo sie sind" (seed, `:130-145`) — `user.name` is a login credential and PK for `user_role`/`user_invitation`/auth; `sales_person.name` is a display name for shiftplans/reports. Collapsing them would either break the auth PK chain or lose the shiftplan-facing name. | Backend computes a display label per entry (username, sales-person name, or both) and puts it in the list DTO — already flagged as the open DTO question in the seed (`:140-145`). |
| Generic "merge any two records" tool exposed in the UI (pick any User row + any SalesPerson row, click Merge) | Feels reusable/future-proof, and superficially matches "these are two halves of one concept" | For three shapes and a handful of staff, a generic merge tool adds a whole new confirmation/undo/conflict-resolution surface for a scenario (accidentally-duplicated roster entries) that isn't the actual reported problem — the reported problem is "same person, two disconnected admin screens", not "duplicate records that need merging". | The seed's actual mechanism — "add login" on a roster-only entry / "add roster entry" on a login-only entry — already covers the real need without a generic merge concept. |

## Feature Dependencies

```
Person aggregate + backfill migration (data-model-first, already scoped in PROJECT.md)
    └──requires──> nothing new (Variante A already decided)

Merged single-list view with shape badge/filter
    └──requires──> Person aggregate (or at minimum a merged read-model/DTO joining user+sales_person)
    └──requires──> Backend-computed display name per entry (Fat Backend, Thin Client)

"Add login" / "Add roster entry" actions on an entry
    └──requires──> Merged single-list view (so the actions have somewhere to appear/redirect from)
    └──requires──> A picker/search over unlinked users or unlinked sales-persons
                       └──requires──> Backend endpoint listing "unlinked" candidates of the other kind
                                          └──enhances──> replaces today's raw-TextInput linking flow entirely

Section-level role gating on merged detail view
    └──requires──> DTO shape that tells FE which sections are visible/editable for the caller
                       └──conflicts with── page-level gating (today's model: whole `/user_details` page is admin-only)

Distinct unlink/delete confirmation copy per shape
    └──enhances──> Merged single-list view + detail view (pure copy/branching, no hard blocking dependency)
```

### Dependency Notes

- **Merged list requires the Person aggregate or an equivalent read-model:** the seed already treats this as settled (`person` table + backfill before UI), so this dependency is not a new discovery — it's confirmation that the feature ordering in the seed (data-model-first, then UI) is the only viable order. A UI-only merge without the aggregate would just be gluing the two existing tabs together with more client-side joining logic, which the current app already half-does (`sales_person_user_links`/`user_sales_person_links` in the store) and which the seed explicitly rejected as insufficient once a real `Person` object was decided.
- **The link/unlink picker requires a new backend "list unlinked X" endpoint:** this doesn't exist today — `SalesPersonDao::find_sales_person_by_user_id` (seed, `:47-49`) finds an existing pairing but there is no "give me all users with no sales_person_id" query. This is new, small, backend work that unlocks replacing the raw-text-field linking flow — call this out explicitly to the roadmapper as a required-but-not-yet-scoped DAO/service addition.
- **Section-level gating conflicts with today's page-level model:** `/user_details` is implicitly admin-territory as a whole page; `/sales_person_details` is implicitly shiftplanner-territory as a whole page. Merging them means the *same* page must show/hide/disable individual sections based on the caller's role — this is a bigger permission-model change than it first looks, and should get its own explicit design decision in discuss-phase rather than being assumed "just render less."

## MVP Definition

### Launch With (v1 — this milestone)

- [ ] `Person` table + backfill migration (already locked/scoped in `PROJECT.md`) — foundation, nothing else works without it
- [ ] One merged list at `/user_management/` (no tabs) with a shape badge/column and a shape filter — this is the literal complaint the milestone exists to fix
- [ ] Backend-computed display name per entry in the list DTO
- [ ] One merged detail component per entry that blends sections in/out by shape (locked decision) with section-level role gating
- [ ] "Add login" / "Add person" as actions on the entry, backed by a real picker (search existing unlinked X) plus a "create new" path — replaces the current raw-`TextInput` flow
- [ ] Honest, shape-aware confirmation copy for unlink and for delete-either-half

### Add After Validation (v1.x)

- [ ] Fold `/employees/:id` (HR report view) into the same entry as a linked/"jump to report" affordance, if users find the two-screen jump (roster admin vs report) annoying in practice — seed already flags this as deliberately deferred pending real usage feedback (`:170-173`)
- [ ] Bulk actions (e.g., select several roster-only volunteers, deactivate together) — only if the roster grows enough that one-at-a-time becomes a real friction point

### Future Consideration (v2+, likely never for this shop size)

- [ ] Automatic link-suggestion / fuzzy-matching — defer indefinitely, wrong scale (Anti-Feature/Differentiator table above)
- [ ] Generic merge tool / bulk-import wizard — defer indefinitely, wrong scale
- [ ] Person-level profile fields (email, phone, DOB) — only relevant once there's an actual product need for storing personal data outside `user`/`sales_person`; seed explicitly keeps `person` identity-only for now

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Person aggregate + backfill | HIGH (blocks everything else) | MEDIUM (migration + service layer, no UI) | P1 |
| Merged single list + shape badge/filter | HIGH (the literal complaint) | LOW–MEDIUM | P1 |
| Backend-computed display name | MEDIUM (small but avoids FE fallback logic per Fat-Backend principle) | LOW | P1 |
| Merged detail component with section gating | HIGH | MEDIUM–HIGH (permission model touch) | P1 |
| Add-login/add-person actions + real picker | HIGH (fixes the worst current UX — raw text field) | MEDIUM | P1 |
| Shape-aware unlink/delete confirmation copy | MEDIUM (safety, not core workflow) | LOW | P1 (cheap, do alongside the above) |
| Fold in `/employees/:id` report view | LOW–MEDIUM (nice-to-have convenience) | MEDIUM | P2 |
| Bulk actions on roster | LOW at current scale | MEDIUM | P3 |
| Auto link-suggestion, generic merge tool, multi-linking, profile fields | LOW at current scale | HIGH | P3 / drop |

**Priority key:**
- P1: Must have for this milestone
- P2: Should have, only if usage after v1 shows real friction
- P3: Nice to have or explicitly out of scope for a shop this size

## Answers to the Specific Research Questions

1. **How is a combined list of heterogeneous entry shapes usually presented?** Badge/column indicating shape + a filter on that same dimension is the dominant pattern (GitLab members list: role/status/pending filters; Salesforce Person Accounts: one record type, shape visible via layout/field). Grouping (separate sub-lists per shape) is what the *current* app already does via tabs and is exactly what the seed wants removed — don't reintroduce grouping as a "compromise," use badge+filter on one flat list instead. Search should span whichever name(s) an entry has, computed server-side.

2. **How is "no login yet" vs "no person yet" surfaced?** Both directions are functionally symmetric badge/label states in every precedent found (GitLab: "invited, no account yet"; the current app: `Unlinked` text). The one asymmetry worth encoding here is *severity of consequence*, not visual novelty: because `sales_person.id` carries billing/shift history and `user.name` carries auth/roles, a "no login" roster entry is low-risk to leave as-is, while a "no roster entry" login (pure admin/HR account) is the actually-intended normal case, not a gap to flag urgently. Badge copy should reflect that ("Volunteer, no login" reads as neutral; "Admin account, no roster entry" should not read as an error state).

3. **How is "add login"/"link existing" usually designed — inline, modal, wizard?** Modal with two explicit, separately-labelled actions ("create new" vs "link existing") is the dominant pattern (Keycloak's admin-UI backlog treats these as two dropdown items precisely to avoid the current app's problem of a single ambiguous text field). A full wizard is overkill for a two-field decision (which existing person? or blank to create new?); inline-on-page editing is fine for already-linked data but not for the search/pick step, which needs its own focused modal with a picker over the unlinked-candidates endpoint.

4. **Expected behaviour around unlinking and deleting one half of a linked pair?** Every precedent that discusses this (Auth0 account-linking guidance) converges on: unlinking must state what the link *did* before removing it — don't just say "unlink?", say what becomes inaccessible/invisible. Deleting one half must not silently touch the other half, and the confirmation copy must differ by which half is being deleted, given the asymmetric blast radius (SalesPerson deletion risk >> User deletion risk in this schema, per the seed's twelve-table fan-out on `sales_person.id`). No precedent found for cascading hard-deletes across the pair as a *table-stakes* expectation — the safe default is soft-delete/deactivate the SalesPerson side (already the existing `inactive` flag) and hard-delete only the User/login side, independently.

5. **Table stakes vs overkill for a shop with a handful of staff?** Table stakes: one flat filterable/searchable list with shape badges, a real picker for linking (not a raw text field), section-level (not page-level) role gating, and honest confirmation copy. Overkill: fuzzy auto-matching, audit/merge-history UI, bulk import, multi-account linking, profile-completeness gamification. The dividing line throughout is scale: everything in the "differentiator/anti-feature" tables solves a problem that only appears with hundreds/thousands of accounts and multiple concurrent admins — neither is true here.

## Sources

- Current codebase (HIGH confidence, read directly): `shifty-dioxus/src/page/user_management.rs`, `shifty-dioxus/src/page/user_details.rs`, `shifty-dioxus/src/page/sales_person_details.rs`, `.planning/seeds/personenverwaltung-zusammenlegen.md`, `.planning/PROJECT.md`, `dao_impl_sqlite/src/sales_person.rs` (grep-verified `sales_person_user` usage).
- [Salesforce Help — What are Person Accounts in Salesforce?](https://www.sigcorp.com/insights/what-are-person-accounts-in-salesforce/) — MEDIUM confidence (vendor-adjacent third-party explainer, cross-checked against Salesforce's own Account Fields doc). Confirms the "one UI record, blended fields by shape" pattern the seed already committed to.
- [Salesforce Help — Account Fields](https://help.salesforce.com/s/articleView?id=account_fields.htm&language=en_US&type=0) — MEDIUM confidence (official vendor docs).
- [Auth0 Docs — User Account Linking](https://auth0.com/docs/manage-users/user-accounts/user-account-linking) — MEDIUM-HIGH confidence (official vendor docs, identity-linking domain authority). Basis for the unlink-confirmation-copy recommendation.
- [PingIdentity — Link Multiple Accounts to a Single Identity](https://docs.pingidentity.com/pingidm/8.1/samples-guide/multi-account-linking.html) — MEDIUM confidence (official vendor docs). Used to confirm multi-account-linking is an enterprise-scale differentiator, not needed here.
- [Keycloak GitHub Issue #45704 — Invite existing users from Admin UI](https://github.com/keycloak/keycloak/issues/45704) — MEDIUM confidence (OSS project issue tracker, direct discussion of the "create new vs link existing" ambiguity problem). Basis for the modal-with-two-explicit-actions recommendation.
- [GitLab Docs — Members of a project](https://docs.gitlab.com/user/project/members/) and [GitLab Work Item #262876 — Filter members list](https://gitlab.com/gitlab-org/gitlab/-/work_items/262876) — MEDIUM-HIGH confidence (official OSS project docs/issue tracker). Basis for the badge+filter recommendation over grouping/tabs, and for the "invited, no account yet" state pattern.
- [GitHub Docs — REST API endpoints for organization members](https://docs.github.com/en/rest/orgs/members) — MEDIUM-HIGH confidence (official vendor docs). Corroborates "pending/no-account" as a first-class filterable state distinct from active members.

---
*Feature research for: merged person/identity administration (v2.7 Personenverwaltung)*
*Researched: 2026-09-16*
