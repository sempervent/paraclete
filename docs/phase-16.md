# Phase 16: Token ergonomics — rotation and `last_used_at`

Phase 16 adds **`last_used_at`** tracking on successful authentication, an **admin-only rotation** API and
CLI, and audit/logging for rotation events.

## 1. Summary

- **Persistence**: `auth_tokens.last_used_at`, `auth_tokens.replaced_by_token_id` (migration
  `20250420120000_auth_tokens_last_used_rotation.sql`).
- **Auth**: `verify_bearer_token` in `crates/paraclete-store/src/auth_store.rs` updates **`last_used_at`**
  on each successful verification.
- **Rotation**: `rotate_auth_token` in the same file — new row + disable old +
  linkage; **`POST /api/v1/admin/tokens/{token_id}/rotate`** returns **`201`** + new secret once.
- **Wire/API**: **`AuthTokenSummaryView`** / list/get include **`last_used_at`** and **`replaced_by_token_id`**;
  **`AuthTokenRotateResponse`** for rotate.
- **CLI**: **`paraclete token rotate <uuid>`**; list/get/JSON show usage fields.
- **TUI**: token list lines include **`last=…`** (RFC 3339 or `-`).
- **OpenAPI**: **0.16.0**; rotate path documented.
- **ADRs**: **0027** (`last_used_at`), **0028** (rotation).

## 2. Files added or changed (representative)

| Area | Files |
|------|--------|
| Store | `migrations/20250420120000_auth_tokens_last_used_rotation.sql`, `auth_store.rs` |
| Service | `api_types.rs`, `service.rs`, `http/handlers.rs`, `http/mod.rs`, `observability.rs`, `openapi/*`, tests |
| CLI | `api.rs`, `client.rs`, `cli.rs`, `cmd.rs`, `render.rs`, `tests/e2e.rs` |
| TUI | `run.rs` |
| Docs | `README.md`, `docs/architecture.md`, `docs/domain-model.md`, `docs/implementation-log.md`, `docs/phase-16.md`, `docs/adr/0027*.md`, `docs/adr/0028*.md`, `mkdocs.yml` |

## 3. Token ergonomics architecture

- **Store**: Single source of truth for hashes, **`last_used_at`**, rotation transaction.
- **HTTP**: Auth middleware calls **`ParacleteService::store().verify_bearer_token`** (via **`AppState`**) — same
  as before; verify now includes the **`UPDATE`**.
- **Admin**: **`ParacleteService::admin_rotate_token`** delegates to **`rotate_auth_token`**; handler emits
  **`audit::token_rotated`**.
- **Clients**: CLI/TUI use **`paraclete_cli::ApiClient`** / shared DTOs; no local token crypto.

## 4. API / CLI changes

| Item | Detail |
|------|--------|
| **`POST /api/v1/admin/tokens/{token_id}/rotate`** | Admin only; empty body; **`201`**, **`AuthTokenRotateResponse`** |
| **List / get** | **`last_used_at`**, **`replaced_by_token_id`** on **`AuthTokenSummaryView`** |
| **CLI** | **`paraclete token rotate <token-id>`**; table adds **`last_used`** column |

## 5. Persistence changes

| Column | Purpose |
|--------|---------|
| **`last_used_at`** | Last successful `verify_bearer_token` (nullable until first use) |
| **`replaced_by_token_id`** | New token id after rotation (null if not rotated) |

## 6. Tests added

- **Store** (`crates/paraclete-store/tests/auth_tokens.rs`): **`verify_sets_last_used_at`**, **`rotate_disables_old_and_mints_new_secret`**.
- **HTTP** (`admin_tokens.rs`): **`admin_rotate_invalidates_old_secret_and_sets_last_used`**, **`operator_cannot_rotate_token`**.
- **CLI e2e**: **`token_rotate_cli_new_secret_works_old_fails`**.
- **OpenAPI**: version **0.16.0**, path **`/rotate`**.

## 7. Validation commands

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
mkdocs build --strict
```

## 8. Deferred items

OIDC/SSO, multi-tenant RBAC, Postgres store, object stores, plugin runtime, CI/CD — unchanged.

## What helps create the next prompt

- **SQLite vs Postgres**: Hardened SQLite remains fine until **HA / multi-writer** pain dominates; Postgres is
  not required for token features completed here.
- **TUI row-selection UX**: Still optional polish; **query/path error normalization** (HTTP) is a smaller
  cross-client fix if API consistency is the priority.
- **Rotation vs future OIDC**: Opaque **token rows** and **rotation linkage** map cleanly to “API keys”; OIDC
  would add **identity subjects** and **session** semantics — keep rotation ADR boundaries when layering SSO.
- **Next phase candidates**: **Postgres** (scale), **TUI UX**, **Query/Path JSON envelope** — pick by ops vs
  developer friction.
- **Awkward workflows**: **Many tokens** in CLI table width; **no** automated secret distribution; **backup**
  of SQLite still operator-owned (Phase 15).
