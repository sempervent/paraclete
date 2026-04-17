//! SQLite-first persistence for Paraclete scan runs (history, projections, integrity checks).
//!
//! The scan engine (`paraclete-core`) produces [`paraclete_types::ScanReport`] values; this crate
//! stores canonical JSON blobs plus normalized tables for listing and diffing without coupling
//! storage to scan logic.

#![forbid(unsafe_code)]

mod error;
mod labels;
mod sqlite_store;

pub use error::StoreError;
pub use paraclete_types::{
    apply_redaction_policy, diff_reports, RedactionPolicy, RetentionPolicy, RunDiff, RunId,
    RunOutcome, ScanRun, ScanRunListItem, StoredReportRef, SummaryDelta, TargetIdentity,
};
pub use sqlite_store::{
    RunPublicMeta, ScanJobRow, SqliteScanStore, StaleRecoveryStats, StoredAssetRow,
    StoredFindingRow,
};
