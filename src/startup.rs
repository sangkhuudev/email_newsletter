use crate::authentication::reject_anonymous_users;
use crate::configuration::{DatabaseSettings, Settings};
use crate::email_client::EmailClient;
use crate::routes::{
    health_check, home, subscribe,
    confirm, publish_newsletter,publish_newsletter_form,
    login, login_form, log_out, admin_dashboard,
    change_password, change_password_form,
};
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer, cookie::Key};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;
use secrecy::{Secret, ExposeSecret};
use actix_web_flash_messages::FlashMessagesFramework;
use actix_web_flash_messages::storage::CookieMessageStore;
use actix_session::SessionMiddleware;
use actix_session::storage::RedisSessionStore;
use actix_web_lab::middleware::from_fn;

//----------------------------------------------------------------
pub struct Application {
    pub port: u16,
    pub server: Server,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, anyhow::Error> {
        let connection_pool = get_connection_pool(&configuration.database)
            .await
            .expect("Failed to connect to Postgres.");
        // let timeout = configuration.email_client.timeout();
        // // Build an `EmailClient` using `configuration`
        // let sender_email = configuration
        //     .email_client
        //     .sender()
        //     .expect("Invalid sender email address");
        // let email_client = EmailClient::new(
        //     configuration.email_client.base_url,
        //     sender_email,
        //     // Pass argument from configuration
        //     configuration.email_client.auth_token,
        //     timeout,
        // );
        let email_client = configuration.email_client.client();
        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(&address)?;
        let port = listener.local_addr().unwrap().port();
        let server = run(
            listener, 
            connection_pool, 
            email_client, 
            configuration.application.base_url,
            configuration.application.hmac_secret,
            configuration.redis_uri, 
        )
        .await?;

        Ok(Self { port, server})
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}
//----------------------------------------------------------------

pub async fn get_connection_pool(configuration: &DatabaseSettings) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .connect_with(configuration.with_db())
        .await
}
//----------------------------------------------------------------
// We need to define a wrapper type in order to retrieve the URL
// in the `subscribe` handler.
// Retrieval from the context, in actix-web, is type-based: using
// a raw `String` would expose us to conflicts
pub struct ApplicationBaseUrl(pub String);

pub async fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
    base_url: String,
    hmac_secret: Secret<String>,
    redis_uri: Secret<String>,
) -> Result<Server, anyhow::Error> {
    // Wrap db_pool into a smart pointer which is Arc
    let db_pool = web::Data::new(db_pool);
    let email_client = web::Data::new(email_client);
    let base_url = web::Data::new(ApplicationBaseUrl(base_url));
    let secret_key =  Key::from(hmac_secret.expose_secret().as_bytes());
    let message_store = CookieMessageStore::builder(secret_key.clone()).build();
    let message_framework = FlashMessagesFramework::builder(message_store).build();
    let redis_store = RedisSessionStore::new(redis_uri.expose_secret()).await?;

    // Capture connection from surrouding environment
    let server = HttpServer::new(move || {
        App::new()
            .wrap(message_framework.clone())
            .wrap(SessionMiddleware::new(
                redis_store.clone(), 
                secret_key.clone()
            ))
            .wrap(TracingLogger::default())
            .route("/", web::get().to(home))
            .service(
                web::scope("/admin")
                    .wrap(from_fn(reject_anonymous_users))
                    .route("/dashboard", web::get().to(admin_dashboard))
                    .route("/newsletters", web::get().to(publish_newsletter_form))
                    .route("/newsletters", web::post().to(publish_newsletter))
                    .route("/password", web::get().to(change_password_form))
                    .route("/password", web::post().to(change_password))
                    .route("/logout", web::post().to(log_out))
            )
            .route("/login", web::get().to(login_form))
            .route("/login", web::post().to(login))
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .route("/subscriptions/confirm", web::get().to(confirm))
            .route("/newsletters", web::post().to(publish_newsletter))
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
            .app_data(base_url.clone())
            .app_data(web::Data::new(HmacSecret(hmac_secret.clone())))
    })
    .listen(listener)?
    .run();

    Ok(server)
}

#[derive(Clone)]
pub struct HmacSecret(pub Secret<String>);