#[tokio::main]
async fn main() -> anyhow::Result<()> {
    lensql::run().await?;
    Ok(())
}
