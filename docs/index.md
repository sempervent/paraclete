# Paraclete documentation

Paraclete is a **Rust-first, engine-centric forensic exploration system** for columnar and
adjacent datasets. It produces **structured findings** backed by **evidence**, not prose
vibes. A **Python plugin layer** extends the engine without becoming the engine.

This site tracks **architecture**, the **domain model**, and **phase boundaries** so later
implementation stays coherent.

## Where to start

- [Architecture](architecture.md) — components, data flow, format tiers
- [Domain model](domain-model.md) — targets, scans, findings, reports, plugins
- [Phase 0](phase-0.md) — what exists today and what is intentionally deferred
- [Phase 1](phase-1.md) — Parquet vertical slice on local disk
- [Phase 4](phase-4.md) — SQLite persistence, run history, diff foundations
- [Phase 5](phase-5.md) — application service + HTTP API MVP
- [Implementation log](implementation-log.md) — living record of decisions and follow-ups

## Building docs locally

```bash
python3 -m venv .venv-docs
source .venv-docs/bin/activate
pip install -r docs/requirements.txt
mkdocs serve
```

The repository root contains `mkdocs.yml`; documentation sources live under `docs/`.
