use email_newsletter::startup::run;
use email_newsletter::configuration::get_configuration;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Panic if we can't read the configuration file
    let configuration = get_configuration().expect("Failed to read configuration file");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;
    run(listener)?.await
}

