# Phase 7 — Job-oriented HTTP CLI

## 1. Summary

The workspace ships **`paraclete-cli`** with a **`paraclete`** binary that is a **thin HTTP client** over **`/api/v1`**: submit async scans, poll jobs, read runs/reports, paginate assets and findings, diff runs, and check health. Every command supports **`--json`** for machine output; defaults are human-oriented tables or short text. **`--base-url`** / **`PARACLETE_BASE_URL`** default to **`http://127.0.0.1:8080`**. The CLI does **not** open SQLite or call **`ScanEngine`** for normal operation.

## 2. Files added or changed (this phase)

- `Cargo.toml` (workspace members + `reqwest`, `clap`, `comfy-table`)
- `crates/paraclete-cli/` (new crate: `lib`, `client`, `cmd`, `cli`, `api`, `wire`, `render`, `error`, `main`, tests)
- `README.md`, `docs/architecture.md`, `docs/domain-model.md`, `docs/implementation-log.md`, `docs/adr/0012-http-first-cli.md`, `mkdocs.yml`

## 3. CLI architecture

- **`ApiClient`** (`reqwest` + **rustls**) performs GET/POST against **`PARACLETE_BASE_URL`**.
- **`wire::StartScanRequest`** matches the service JSON body for scan submission.
- **`api::*`** types **Deserialize/Serialize** for responses and **`--json`** printing.
- **`render`** builds **comfy-table** tables for list commands when **`--json`** is unset.
- **`main`** uses **`Cli::parse()`** so **`--help` / `--version`** follow clap’s normal exit; **`run_from_args`** remains for tests.

## 4. Command surface

| Command | Role |
|--------|------|
| `paraclete health` | `GET /api/v1/health` |
| `paraclete scan submit` | `--file` \| `--directory` \| `--target-json`, `--profile`, `--max-files` → `POST /api/v1/jobs/scans` |
| `paraclete job get <id>` | `GET /api/v1/jobs/{id}` |
| `paraclete job wait <id>` | Poll `job get` until terminal; **`--interval-ms`**, **`--timeout-secs`** |
| `paraclete job list` | `GET /api/v1/jobs` (`--status`, `--limit`, `--offset`) |
| `paraclete run get <id>` | Run summary |
| `paraclete run report <id>` | Full report JSON; **`--output`** file |
| `paraclete run assets` / `findings` | Paginated projections |
| `paraclete run list` | `--target-kind`, `--normalized-key`, `--limit` |
| `paraclete diff <a> <b>` | `GET /api/v1/diff` |

Global: **`--base-url`**, **`--json`**.

## 5. Exit codes

- **`0`** — command completed successfully (`job wait` only exits 0 on **succeeded**).
- **`1`** — API/store error, failed job, I/O error, or other runtime failure.
- **`2`** — clap usage error (e.g. missing required subcommand when using `run_from_args` in tests).

## 6. Tests

- **`cli_parse`**: help output, missing subcommand, global `--json` with subcommand help.
- **`api_decode`**: stable error envelope JSON.
- **`e2e`**: in-process Axum + temp SQLite; subprocess `paraclete` via **`spawn_blocking`**; submit → wait → run get; diff two runs; run not found nonzero. **`TempDir`** kept alive for the server lifetime.

## 7. Validation commands

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
mkdocs build
```

## 8. Deferred (intentionally out of scope)

- TUI; local engine/store shortcuts; auth; cancellation; CI/CD; plugin runtime.

## What helps create the next prompt

- **TUI**: Reasonable next step once operators want interactive browsing; same HTTP client types can back a TUI crate.
- **Stale-job recovery** should land **before** investing in rich terminal UX—otherwise polling/wait semantics stay misleading after crashes.
- **Fragile assumptions**: single default base URL; no TLS pinning; error envelope only when the service returns JSON errors (Axum parse failures still differ).
- **Next phase candidates**: **stale-job recovery** + worker hardening, **auth** for remote hosts, **TUI** over the same API, or **OpenAPI/client codegen** if many consumers appear.
- **Awkward workflows**: large reports only to stdout/file without streaming; no combined “submit and tail” beyond `job wait`; diff output is JSON-only for machines (`--json` compact vs human pretty for diff).
