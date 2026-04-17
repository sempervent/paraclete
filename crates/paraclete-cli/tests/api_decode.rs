//! JSON decoding for API error envelopes.

use paraclete_cli::api::ErrorBody;

#[test]
fn parses_error_envelope() {
    let s = r#"{"error":{"code":"run_not_found","message":"run not found","details":{}}}"#;
    let b: ErrorBody = serde_json::from_str(s).unwrap();
    assert_eq!(b.error.code, "run_not_found");
}
