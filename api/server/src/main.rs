use hausmeister::settings::read_config;

mod trace;

async fn start_up() -> color_eyre::Result<()> {
    // We don't care if we can't read the dotenv
    let _ = dotenv::dotenv();

    trace::setup()?;

    let config = read_config()?;

    let (_, app) = hausmeister::create_app(config).await?;
    app.await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    color_eyre::install().unwrap();
    if let Err(err) = start_up().await {
        println!("{:?}", err);
    }
}
