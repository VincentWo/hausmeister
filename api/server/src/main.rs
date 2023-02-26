use hausmeister::settings::read_config;

mod trace;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv()?;

    color_eyre::install()?;
    trace::setup()?;

    let config = read_config()?;

    let (_, app) = hausmeister::create_app(config).await?;
    app.await?;
    Ok(())
}
