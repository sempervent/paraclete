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

**Phase 8** hardens **async scan jobs**: **lease + heartbeat** metadata on **`scan_jobs`**,
**bounded requeue** or **`job_retries_exhausted`** when a worker dies, and **continuous stale
recovery** in the in-process worker loop (still **no external queue**). See **`docs/phase-8.md`**.

**Phase 7** added **`paraclete-cli`** (`paraclete`): a thin HTTP client over **`/api/v1`**
(**`docs/phase-7.md`**). Underlying: **Phase 6** jobs + worker (**`docs/phase-6.md`**), **Phase 5** HTTP,
**Phase 4** SQLite.

**Run the HTTP server**

```bash
export PARACLETE_DATABASE_URL="sqlite://$(pwd)/target/paraclete-dev.sqlite"
cargo run -p paraclete-service --bin paraclete-http
```

**Run the CLI** (with the server reachable at `PARACLETE_BASE_URL`)

```bash
cargo run -p paraclete-cli --bin paraclete -- health
cargo run -p paraclete-cli --bin paraclete -- scan submit --file /path/to/file.parquet --profile standard
```

## Workspace structure

```text
Cargo.toml                 # Rust workspace root
crates/
  paraclete-types/         # Durable JSON-friendly contracts
  paraclete-core/          # ScanEngine, Parquet inspection, shallow text, built-in rules
  paraclete-store/         # SQLite scan runs: migrations, blob + projections, list + diff helpers
  paraclete-service/       # ParacleteService + Axum /api/v1 HTTP + OpenAPI
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
