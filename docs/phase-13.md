# Phase 13: TUI over the HTTP API

Phase 13 adds **`paraclete-tui`**, a **ratatui** + **crossterm** operator console that speaks **only**
to the existing **`/api/v1`** surface via **`paraclete_cli::ApiClient`** — the same types and HTTP
semantics as the **`paraclete`** CLI. There is **no** SQLite, engine, or store access from the TUI.

## What works

- **Connection**: base URL + bearer token (`PARACLETE_BASE_URL` / `PARACLETE_TOKEN` pre-fill), **Enter**
  runs **health** + **whoami**.
- **Home**: menu for jobs, scan submit, runs-for-target, run summary, projections (assets/findings),
  diff, token admin, refresh.
- **Jobs**: list recent jobs; **Job watch** with UUID buffer, periodic poll, terminal-state detection.
- **Submit scan**: local file or directory (`t`), profile cycle (`y`), Tab cycles path/profile/max_files,
  **Enter** submits async job → job watch.
- **Runs for target**: kind + key (`k`/`n`), **Enter** loads list.
- **Run summary**: UUID + **l** loads summary; sets **selected run** for projections.
- **Projections**: paginated assets/findings (`n`/`p`), tab switch.
- **Diff**: two run UUIDs (`[`/`]` focus), **Enter** loads `RunDiff` summary.
- **Tokens** (admin): list, create (`c`), disable (`d`) with field focus (`x`).

Polling uses a **250 ms** tick; job watch uses **`watch_poll_ms`** (default **1000 ms**).

## Validation

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
mkdocs build --strict
```

## Deferred

Direct engine/store, OIDC, rich charts, local token persistence, clipboard.

## What helps create the next prompt

- **Persistence / HA** is still the scale ceiling; the TUI does not change that.
- **Token rotation / `last_used_at`** remain API/product work; TUI can grow with those endpoints.
- **Fragile assumptions**: large run reports are not loaded in full (summary only); JSON-body parse errors
  are normalized in **Phase 14** (`invalid_json_request` envelope).
- **Next phase candidates**: Postgres/HA, token ergonomics — pick by ops pain.
- **Awkward workflows**: typing UUIDs in the TUI; consider future “pick from list” flows when APIs allow.
