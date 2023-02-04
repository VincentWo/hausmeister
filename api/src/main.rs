#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    hausmeister::run().await?;

    Ok(())
}
