# StatePR-02 — greentic-state: Conservative alignment + fix gaps without regressions

## Repo
`greentic-state`

## Goal
`greentic-state` is the **existing** state implementation and should remain the reference semantics/backing store for hosts.
This PR should be **conservative**:
- Fix inconsistencies, missing behaviors (especially delete / prefix deletion / TTL semantics), and doc gaps.
- Add regression tests around current behavior.
- Provide small “glue” helpers if they already exist in similar form, but avoid major refactors.

## Non-goals
- Do NOT redesign the state layer or introduce “yet another” incomplete state API.
- Do NOT change public APIs in a way that breaks downstream repos.
- Do NOT add a new backend unless the repo already has multiple backends and this PR is simply fixing one.

---

## Work Items

### 1) Document current semantics (without changing behavior)
Add/update a doc (e.g. `docs/component_state.md`) that describes **what the repo already does**:

- Scoping: `TenantCtx` (env + tenant + optional team/user)
- Namespacing: `prefix` (flow namespace)
- Keying: `StateKey`
- Optional TTL and how it is applied/preserved
- JSON helpers: `set_json`, `get_json`, and JSON Pointer paths **if they exist today**

This doc must clearly state:
- Components (Wasm guests) do NOT import `greentic-state`.
- Hosts/runners use `greentic-state` as backing for WIT `greentic:state/store@1.0.0`.

### 2) Audit and fix correctness gaps (minimal diffs)
- Verify `delete` semantics exist and behave correctly.
- Verify `del_prefix` semantics exist and do not leak across tenants.
- Verify TTL behavior is consistent with existing docs/comments:
  - If `None` TTL preserves existing TTL, ensure that behavior is true and tested.
  - If TTL behavior differs, update docs to reflect reality (prefer “docs match code” unless behavior is clearly buggy).

Make small targeted fixes only where behavior is clearly wrong or inconsistent.

### 3) Add regression tests (high priority)
Add tests that lock down current expected behavior:
- Write -> Read roundtrip (bytes and JSON helpers if supported)
- Update semantics (write same key updates)
- Delete (read returns None after delete)
- Prefix delete (only affects keys under that prefix and tenant)
- TenantCtx isolation (same prefix+key in different tenants must not collide)
- TTL tests (if TTL supported): expiry/preservation semantics

### 4) Optional: tiny host “adapter glue” only if already present in pattern
If this repo already contains (or downstream expects) wiring helpers for hosts, you may:
- Add a small module that helps a host map WIT state-store calls into the existing `greentic-state` store trait.
But:
- Do not restructure crates.
- Do not create new traits that compete with existing ones.
- Prefer implementing adapters in the runner-host if that is where wiring already lives.

---

## Acceptance Criteria
- No major API churn; minimal diffs.
- Docs reflect actual behavior.
- Regressions are prevented by tests.
- Missing/incorrect delete or prefix-delete behaviors are fixed (if they are indeed broken today).

## Notes for Codex
- Treat this as “tighten + test”, not “re-architect”.
- If you discover downstream dependencies on undocumented quirks, document them and test them rather than changing them.
