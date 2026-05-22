#[tokio::main]
async fn main() -> anyhow::Result<()> {
    snap_rat::run().await
}
