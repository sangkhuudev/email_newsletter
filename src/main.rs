use email_newsletter::telemetry::{get_subscriber, init_subscriber};
use email_newsletter::configuration::get_configuration;
use email_newsletter::startup::Application;

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    let subscriber = get_subscriber("email_newsletter".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);
    // Panic if we can't read the configuration file
    let configuration = get_configuration().expect("Failed to read configuration file");
    let application = Application::build(configuration).await?;
    application.run_until_stopped().await?;
    Ok(())
}

