# Phase 0 — scaffold and contracts

Phase 0 optimizes for **a correct skeleton**: explicit seams, typed contracts, tests, and
documentation that later code can extend without surgery.

## Established in Phase 0

- Cargo **workspace** with four crates and clear responsibilities
- Shared **domain model** (`paraclete-types`) covering targets, scans, datasets, findings,
  and reports
- Engine **traits** (`paraclete-core`) for resolution, detection, inspection, rules, and
  plugin execution, plus a minimal `ScanOrchestrator`
- Report helpers (`paraclete-report`) for JSON + Markdown stub + validation re-exports
- Rust **plugin protocol** structs + `PluginExecutor` trait (`paraclete-plugin-protocol`)
- Python **Pydantic contracts** + dev tooling (`uv`, `ruff`, `pytest`)
- **Fixtures** for CSV, JSON, Parquet (tiny), reports, and a sample plugin manifest
- **MkDocs Material** site under `docs/` with ADRs and an implementation log
- **Quality gates** documented in the root `README.md` (`fmt`, `clippy -D warnings`,
  `test`, Python lint/format/test)

## Intentionally deferred

- HTTP servers, REST/GraphQL designs, authentication, pagination
- CLI / TUI and user-facing argument parsing (`clap` is listed as future-only)
- Real dataset inventory, metadata mining, Parquet footer parsing, CSV sniffing
- Rule catalog beyond taxonomy + codes + trait seam
- Plugin process management, sandboxing, versioning policy for wheels, and result
  streaming
- CI/CD workflows (GitHub Actions, release automation) — repo is structured so adding them
  is mechanical
- Object store credentials, listing APIs, and incremental scan persistence

## Near-term handoff to Phase 1

Phase 1 should pick a **single vertical slice** through the engine:

1. Real **target resolution** for local directories with safe enumeration caps.
2. A **Parquet adapter** that can read file layout + schema + row-group sizes using the
   ecosystem’s Parquet crate, emitting at least one non-trivial finding with evidence.
3. Wire results into `ScanReport` with validated summaries.

Parallel work can extend CSV/JSON paths, but Parquet-first depth is the architectural
priority (see ADR 0003).
