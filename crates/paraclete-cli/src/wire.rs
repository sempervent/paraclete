//! Wire types matching `paraclete-service` HTTP JSON (`StartScanRequest`).

use paraclete_types::{RedactionPolicy, ScanOptions, ScanProfile, ScanTarget};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartScanRequest {
    pub target: ScanTarget,
    pub profile: ScanProfile,
    #[serde(default)]
    pub options: ScanOptions,
    #[serde(default)]
    pub scan_id: Option<Uuid>,
    #[serde(default)]
    pub redaction: Option<RedactionPolicy>,
}
