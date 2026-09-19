# Paraclete

Paraclete is a **Rust-first, engine-centric forensic exploration system** for columnar and
adjacent datasets. It is being built to inspect real collections of files, infer how they
behave as logical datasets, and emit **structured findings** backed by **concrete
evidence**—not unstructured prose.

## Why it exists

Data lakes and local file collections often fail quietly: partition keys drift, schemas
fork, metadata lies, and tiny files accumulate until performance or correctness collapses.
Paraclete exists to make those failures **legible, evidenced, and comparable across
scans**.

## Current phase

**Phase 17** adds a **Postgres-backed** store (**`PostgresScanStore`**) alongside SQLite, selected by **`PARACLETE_DATABASE_URL`**
(`postgres://` / `postgresql://` vs `sqlite://`). The service uses **`StoreBackend`**; job claiming on Postgres uses
**`FOR UPDATE SKIP LOCKED`**. See **`docs/phase-17.md`** and ADRs **0029**–**0030**.

**Phase 16** adds **`last_used_at`** on successful bearer verification, **`POST /api/v1/admin/tokens/{token_id}/rotate`**
(admin-only, new secret once; old row disabled with **`replaced_by_token_id`**), CLI **`paraclete token rotate`**, and
audit **`token.rotated`**. See **`docs/phase-16.md`** and ADRs **0027**–**0028**.

**Phase 15** hardens **SQLite** for single-node use: **WAL**, **`synchronous=NORMAL`**, **`foreign_keys=ON`**,
**busy timeout**, and an explicit **`SqliteStoreConfig`** / **`connect_with_config`** path. ADRs **0025**–**0026**
and **`docs/postgres-sqlite-compatibility.md`** document Postgres-oriented notes without a database switch yet.
See **`docs/phase-15.md`**.

**Phase 14** normalizes **JSON request-body** failures to the standard **`ErrorBody`** with
**`invalid_json_request`** via a shared **`ApiJson`** extractor (OpenAPI **0.14.0**). See **`docs/phase-14.md`**
and amended ADR **0022**.

**Phase 13** adds **`paraclete-tui`**: a **ratatui** operator console over **`/api/v1`** only, reusing
**`paraclete_cli::ApiClient`** (same HTTP truth as the CLI). Jobs, runs, projections, diff, and basic
token admin are reachable from the terminal without a second control plane. See **`docs/phase-13.md`**
and ADRs **0023**–**0024**.

**Phase 12** deepens the **OpenAPI 3.1** contract (**0.12.0**, superseded by **0.14.0** in Phase 14): **`utoipa::ToSchema`** / **`IntoParams`**
on wire DTOs and shared types, **`utoipa::path`** stubs for every route, documented **`bearerAuth`**,
standard **error** envelope components, and **examples** on load-bearing responses. See **`docs/phase-12.md`**
and ADRs **0021**–**0022**.

**Phase 11** added **admin-only token lifecycle**: **`POST/GET /api/v1/admin/tokens`**, **`GET`** / **`POST …/disable`**
for **`/api/v1/admin/tokens/{token_id}`**, one-time **`token_secret`** on create, and CLI **`paraclete token`**
(**`create`**, **`list`**, **`get`**, **`disable`**). See **`docs/phase-11.md`** and ADRs **0019**–**0020**.

**Phase 10** adds **Prometheus metrics** (**`GET /metrics`**), **structured audit logging** (`tracing`
target **`paraclete_audit`**), **request IDs** (**`X-Request-Id`**), and **worker/job** metrics and
spans. See **`docs/phase-10.md`** and ADRs **0017**–**0018**.

**Phase 9** added **Bearer API token** authentication: tokens are **hashed at rest** (SQLite or Postgres), validated in
**middleware**, and mapped to **reader / operator / admin** roles. **`GET /api/v1/health`** stays
unauthenticated; everything else under **`/api/v1`** requires **`Authorization: Bearer …`**. Bootstrap
the first token with **`PARACLETE_BOOTSTRAP_TOKEN`** (see **`docs/phase-9.md`**).

Prior: **Phase 8** job recovery (**`docs/phase-8.md`**), **Phase 7** CLI (**`docs/phase-7.md`**).

**Run the HTTP server**

```bash
export PARACLETE_DATABASE_URL="sqlite://$(pwd)/target/paraclete-dev.sqlite"
# Or Postgres, e.g.:
# export PARACLETE_DATABASE_URL="postgres://USER:PASS@127.0.0.1:5432/paraclete"
export PARACLETE_BOOTSTRAP_TOKEN="your-long-random-secret"
cargo run -p paraclete-service --bin paraclete-http
```

**Run the CLI** (set **`PARACLETE_BASE_URL`** and the same secret as **`PARACLETE_TOKEN`** or **`--token`**)

```bash
export PARACLETE_TOKEN="your-long-random-secret"
cargo run -p paraclete-cli --bin paraclete -- health
cargo run -p paraclete-cli --bin paraclete -- who-am-i
cargo run -p paraclete-cli --bin paraclete -- token list
cargo run -p paraclete-cli --bin paraclete -- scan submit --file /path/to/file.parquet --profile standard
```

**Run the TUI** (same env as CLI: **`PARACLETE_BASE_URL`**, **`PARACLETE_TOKEN`**)

```bash
cargo run -p paraclete-tui --bin paraclete-tui
```

## Workspace structure

```text
Cargo.toml                 # Rust workspace root
crates/
  paraclete-types/         # Durable JSON-friendly contracts
  paraclete-core/          # ScanEngine, Parquet inspection, shallow text, built-in rules
  paraclete-store/         # SQLite + Postgres: migrations, blob + projections, jobs, auth, StoreBackend
  paraclete-service/       # ParacleteService + Axum /api/v1 HTTP + OpenAPI
  paraclete-cli/           # paraclete CLI + shared ApiClient library
  paraclete-tui/           # paraclete-tui terminal UI (HTTP client only)
  paraclete-report/        # Report JSON + Markdown stub + validation helpers
  paraclete-plugin-protocol/ # Rust ↔ Python plugin boundary types
python/paraclete_plugins/  # Pydantic contracts + future plugin packages
fixtures/                  # Phase 1–3 Parquet suites + CSV/JSON/report samples
docs/                      # MkDocs sources + ADRs + implementation log
scripts/                   # Minimal helper scripts (fixture generation)
mkdocs.yml                 # Documentation site configuration
```

## Local development

### Rust

Formatting, linting, and tests (matches intended CI posture):

```bash
export PATH="/opt/homebrew/bin:$PATH"   # macOS Homebrew rustc/cargo, if needed
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo run -p paraclete-service --bin paraclete-http   # needs PARACLETE_DATABASE_URL
cargo run -p paraclete-cli --bin paraclete -- --help
cargo run -p paraclete-tui --bin paraclete-tui
```

### Python (`uv`)

```bash
cd python/paraclete_plugins
uv sync --group dev
uv run ruff check .
uv run ruff format --check .
uv run pytest
```

### Docs (MkDocs Material)

```bash
python3 -m venv .venv-docs
source .venv-docs/bin/activate
pip install -r docs/requirements.txt
mkdocs serve
```

### Regenerating Parquet fixtures

```bash
cd python/paraclete_plugins
uv sync --group dev
uv run python ../../scripts/generate_parquet_fixture.py
uv run python ../../scripts/build_phase1_fixtures.py
uv run python ../../scripts/build_phase2_fixtures.py
uv run python ../../scripts/build_phase3_fixtures.py
```

### Updating the golden `ScanReport` JSON fixture

```bash
export PARACLETE_UPDATE_FIXTURES=1
cargo test -p paraclete-report minimal_report_fixture_roundtrips -- --exact
unset PARACLETE_UPDATE_FIXTURES
```

## Roadmap (near term)

- **TUI / richer UX** on top of the same HTTP job API; **stale-job recovery** and worker semantics before heavy UX.
- **Plugins:** concrete Python bridge with sandboxing and versioning policy.
- **Hosting hardening:** auth, richer OpenAPI, distributed workers when needed.

## License

See `LICENSE` (MIT).
