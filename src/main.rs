use email_newsletter::startup::run;
use email_newsletter::telemetry::{get_subscriber, init_subscriber};
use email_newsletter::configuration::get_configuration;
use std::net::TcpListener;
use sqlx::PgPool;
use secrecy::ExposeSecret;
#[tokio::main]
async fn main() -> std::io::Result<()> {

    let subscriber = get_subscriber("email_newsletter".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);
    // Panic if we can't read the configuration file
    let configuration = get_configuration().expect("Failed to read configuration file");
    let connection_pool = PgPool::connect(&configuration.database.connection_string().expose_secret())
        .await
        .expect("Failed to connect to Postgres database");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool)?.await
}

