# Paraclete — Product Vision

## Working title
**Paraclete**

## Tagline
**A forensic exploration system for columnar and adjacent data.**

## One-sentence vision
Paraclete is an API-first, engine-centric data exploration and diagnostics system that inspects Parquet-first datasets, treats CSV/JSON and other formats as secondary citizens, and produces structured findings, evidence, profiles, and remediation guidance for humans, tools, and downstream dashboards.

## Why this exists
Modern data lakes and local dataset collections rot in silence. Partition schemes drift. Schemas fork. Metadata lies. Tiny files metastasize. Temporal extents become folklore. Null patterns explode. Compression choices decay into ritual. Most teams discover these failures only after performance collapses, analyses diverge, or downstream pipelines begin hallucinating certainty.

Paraclete exists to turn that black box into an examinable system.

It is not just a linter. It is not just a metadata reader. It is not just a catalog. It is a **forensic data exploration engine** that can inspect a dataset, infer its shape and conventions, compare reality to expectation, surface anomalies, and explain what it found with concrete evidence.

## Core principles
1. **Parquet-first, not Parquet-only**  
   Parquet is the primary target and receives the deepest support. CSV, JSON, newline-delimited JSON, and later formats may be inspected through a common abstraction, but with shallower guarantees and fewer diagnostics at first.

2. **Engine first**  
   The core engine and contracts come before transport layers. The system must be usable as a Rust library before it becomes an HTTP service, CLI, or TUI.

3. **Structured findings, not blob prose**  
   Every issue must be represented as a typed finding with severity, category, evidence, locations, and recommendations. Human-readable summaries are an output view, not the internal truth.

4. **Evidence over vibes**  
   All findings must point to specific supporting evidence: files, partitions, columns, schema fragments, row-group metadata, sampled values, inferred extents, or rule outputs.

5. **Exploration, not only validation**  
   The system should help users discover what a dataset is, not merely judge whether it is “correct.”

6. **Hybrid extensibility**  
   The performance-critical engine and core rule execution live in Rust. A Python plugin layer allows user-authored rules, enrichers, or analytical extensions without compromising the core architecture.

7. **Local-first and service-capable**  
   The system should work against local files immediately and evolve cleanly toward object stores, persistent scan history, and eventually HTTP and dashboard-backed workflows.

## What Paraclete does
Paraclete scans files, directories, and later object-store prefixes and logical datasets. It inspects structure, metadata, schema, partitioning, statistics, and selected content samples. It then builds a model of the dataset and emits:

- dataset inventory
- inferred logical datasets
- partition scheme analysis
- schema drift analysis
- metadata quality findings
- file-size and row-group heuristics
- temporal/spatial extent truth checks where inferable
- structured findings and evidence
- optional recommendations and remediation hints
- machine-readable scan reports and profiles

## Primary users
- data platform engineers
- geospatial and analytical systems engineers
- lakehouse operators
- pipeline authors
- analysts dealing with half-governed file collections
- developers building observability or catalog tooling on top of dataset inspection

## Scope of support
### First-class
- Parquet files
- Parquet dataset directories
- Hive-style and non-Hive partition patterns
- Local filesystem targets

### Second-class, early
- CSV
- JSON
- NDJSON / JSONL

### Later
- object stores
- Arrow / IPC / Feather
- Delta/Iceberg/Hudi-adjacent inspection surfaces
- catalog integrations
- continuous monitoring
- dashboard-oriented backends

## Product shape
Paraclete will evolve in layers:

1. **Core engine and contracts**
2. **Local execution surfaces**
   - Rust-native command integration
   - later CLI
   - later TUI
3. **Python plugin system**
4. **Persistence and scan history**
5. **HTTP API**
6. **Dashboard/backend integrations**
7. **Automation, scheduling, and CI/CD**

## Architectural stance
Paraclete is a Rust workspace with a core engine crate responsible for:
- scan planning
- format detection
- format adapters
- dataset modeling
- finding generation
- evidence capture
- output contracts
- plugin invocation boundaries

Python is not the engine. Python is an extension membrane.

## Output philosophy
Paraclete should emit stable, versioned, machine-readable reports that can be:
- rendered to human-readable summaries
- diffed across scans
- stored for history
- consumed by external systems
- surfaced later in APIs, dashboards, and automated workflows

## Definition of success
A user points Paraclete at a messy dataset collection and receives a report that answers:
- what files and formats are here?
- what logical datasets exist?
- how are they partitioned?
- what is the schema reality?
- what drift or anomalies exist?
- what evidence supports those conclusions?
- what should be fixed first?

If it can do that clearly, reproducibly, and fast, the machine is alive.