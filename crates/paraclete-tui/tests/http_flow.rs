//! Hermetic `ApiClient` flows against in-process Axum (same client as TUI).

use std::time::Duration;

use paraclete_cli::ApiClient;
use paraclete_service::{build_router, ParacleteService};
use paraclete_store::SqliteScanStore;
use paraclete_types::AuthRole;

const TUI_TEST_TOKEN: &str = "paraclete-tui-http-test-token";

fn sqlite_url(dir: &tempfile::TempDir) -> String {
    let p = dir.path().join("t.sqlite");
    std::fs::File::create(&p).unwrap();
    let abs = p.canonicalize().unwrap();
    format!("sqlite://{}", abs.display())
}

async fn spawn_test_server() -> (tokio::task::JoinHandle<()>, String, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let store = SqliteScanStore::connect(&sqlite_url(&dir)).await.unwrap();
    store.insert_auth_token("tui", TUI_TEST_TOKEN, AuthRole::Operator).await.unwrap();
    let app = build_router(ParacleteService::new(store));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    tokio::time::sleep(Duration::from_millis(80)).await;
    let base = format!("http://{}", addr);
    (server, base, dir)
}

#[tokio::test]
async fn client_health_whoami_list_jobs() {
    let (_srv, base, _dir) = spawn_test_server().await;
    let c = ApiClient::new(&base, Some(TUI_TEST_TOKEN.into())).unwrap();
    let h = c.health().await.unwrap();
    assert_eq!(h.status, "ok");
    let w = c.whoami().await.unwrap();
    assert_eq!(w.label, "tui");
    let jobs = c.list_jobs(None, 10, 0).await.unwrap();
    assert_eq!(jobs.total, 0);
}

#[tokio::test]
async fn client_run_not_found_error() {
    let (_srv, base, _dir) = spawn_test_server().await;
    let c = ApiClient::new(&base, Some(TUI_TEST_TOKEN.into())).unwrap();
    let id = uuid::Uuid::new_v4();
    let e = c.get_run_summary(id).await.expect_err("missing run");
    let msg = e.to_string();
    assert!(msg.contains("run_not_found") || msg.contains("not found"), "{msg}");
}
