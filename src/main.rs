use email_newsletter::startup::run;
use email_newsletter::configuration::get_configuration;
use std::net::TcpListener;
use sqlx::PgPool;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Panic if we can't read the configuration file
    let configuration = get_configuration().expect("Failed to read configuration file");
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres database");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool)?.await
}

