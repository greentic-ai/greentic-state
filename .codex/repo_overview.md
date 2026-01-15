# Repository Overview

## 1. High-Level Purpose
- `greentic-state` provides a multi-tenant JSON state store used by Greentic runtimes, with pluggable backends for in-memory and Redis storage.
- It exposes a `StateStore` trait for whole-document and JSON Pointer-based partial updates, plus key scoping utilities built on `greentic-types`.

## 2. Main Components and Functionality
- **Path:** `src/lib.rs`
  - **Role:** crate entrypoint and re-exports for public API.
  - **Key functionality:** exposes `StateStore`, key utilities, and re-exports `StateKey`, `StatePath`, `TenantCtx`.
  - **Key dependencies / integration points:** `greentic-types`, optional `redis` backend via feature flag.
- **Path:** `src/store.rs`
  - **Role:** defines the `StateStore` trait.
  - **Key functionality:** `get_json`, `set_json`, `del`, `del_prefix`, with TTL semantics (`None` preserves existing TTL).
- **Path:** `src/inmemory.rs`
  - **Role:** in-memory `StateStore` implementation using `DashMap`.
  - **Key functionality:** JSON read/write, JSON Pointer updates, lazy TTL expiry, scoped prefix deletes.
  - **Key dependencies / integration points:** uses `crate::key` for FQN keys and `crate::util` for JSON path handling.
- **Path:** `src/redis_store.rs`
  - **Role:** Redis-backed `StateStore` implementation.
  - **Key functionality:** JSON read/write with Lua upsert for TTL preservation/reset/clear, Redis `SCAN` + `DEL` for prefix deletes.
  - **Key dependencies / integration points:** `redis` crate, Lua script for atomic upserts.
- **Path:** `src/key.rs`
  - **Role:** key scoping and FQN generation.
  - **Key functionality:** derives fully-qualified keys from `TenantCtx`, prefix, and `StateKey`; tenant scope includes env/tenant/team/user.
- **Path:** `src/util.rs`
  - **Role:** JSON Pointer-style access helpers.
  - **Key functionality:** get/set at `StatePath`, auto-creates intermediate arrays/objects, validates indices.
- **Path:** `src/error.rs`
  - **Role:** shared error helpers.
  - **Key functionality:** wraps serde/redis errors and standardizes Greentic error codes.
- **Path:** `tests/*.rs`
  - **Role:** regression coverage for JSON roundtrip, delete semantics, prefix delete, tenant isolation, and TTL behavior.
  - **Key functionality:** verifies in-memory and optional Redis behavior (skips Redis when `REDIS_URL` is absent).
- **Path:** `docs/component-state.md`
  - **Role:** usage guide for component state semantics.
  - **Key functionality:** documents naming/scoping, JSON helpers, delete semantics, and TTL preservation.
- **Path:** `examples/quickstart.rs`
  - **Role:** minimal usage example (in-memory store).
- **Path:** `ci/local_check.sh`
  - **Role:** local CI wrapper for fmt/clippy/test/publish checks.

## 3. Work In Progress, TODOs, and Stubs
- No TODO/FIXME/XXX markers or stubbed implementations found in the repository.

## 4. Broken, Failing, or Conflicting Areas
- **Location:** `ci/local_check.sh` (tests with Redis)
  - **Evidence:** Docker daemon access denied (`/var/run/docker.sock` permission error) when attempting to start Redis.
  - **Likely cause / nature of issue:** Local environment lacks permission to access Docker socket, so Redis-backed tests cannot run via the script.
- **Location:** `ci/local_check.sh` (dependency sanity)
  - **Evidence:** Cargo registry unpack fails with `Permission denied (os error 13)` under `/home/remote/.cargo/registry/src`.
  - **Likely cause / nature of issue:** Local environment permissions prevent writing to the cargo registry cache.
- No documented broken areas or conflicting implementations found.

## 5. Notes for Future Work
- Keep docs aligned with actual TTL and prefix-delete semantics as backends evolve.
- Expand tests if new backends or additional state helpers are added.
