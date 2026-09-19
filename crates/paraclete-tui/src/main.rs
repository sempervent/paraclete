//! `paraclete-tui` — operator console over the Paraclete HTTP API.

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    paraclete_tui::run().await.map_err(|e| e.to_string().into())
}
