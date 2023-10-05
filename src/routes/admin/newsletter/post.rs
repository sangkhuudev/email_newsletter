// Adapt POST /newsletters to process the form data:
// – Change the route to POST /admin/newsletters;
// – Migrate from ‘Basic’ to session-based authentication;
// – Use the Form extractor (application/x-www-form-urlencoded) instead of the Json ex-
// tractor (application/json) to handle the request body;
// – Adapt the test suite.
use actix_web::{web, HttpResponse};
use actix_web_flash_messages::FlashMessage;
use crate::authentication::UserId;
use crate::email_client::EmailClient;
use crate::utils::{e500, see_other};
use crate::domain::SubscriberEmail;
use anyhow::Context;
use sqlx::PgPool;

#[derive(serde::Deserialize)]
pub struct FormData {
    title: String,
    text_content: String,
    html_content: String,
}

#[tracing::instrument(
    name = "Publish a newsletter issue",
    skip(form, db_pool, email_client, user_id),
    fields(user_id=%*user_id)
)]
pub async fn publish_newsletter(
    form: web::Form<FormData>,
    user_id: web::ReqData<UserId>,
    db_pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
) -> Result<HttpResponse, actix_web::Error> {
    let subscribers = get_confirmed_subscribers(&db_pool).await.map_err(e500)?;
    for subscriber in subscribers {
        match subscriber {
            Ok(subscriber) => {
                email_client
                    .send_email(
                        &subscriber.email,
                        &form.title, 
                        &form.html_content, 
                        &form.text_content
                    )
                    .await
                    .with_context(|| {
                        format!("Failed to send newsletter issue to {}", subscriber.email)
                    })
                    .map_err(e500)?;
            }
            Err(error) => {
                tracing::warn!(
                    error.cause_chain = ?error,
                    error.message = %error,
                    "Skipping a confirmed subscriber. There stored contact details are invalid"
                );
            }
        }
    }
    FlashMessage::info("The newsletter issue has been published").send();
    Ok(see_other("/admin/newsletters"))

}

struct  ConfirmedSubscriber {
    email: SubscriberEmail,
}
#[tracing::instrument(
    name = "Get confirmed subscribers",
    skip(db_pool)
)]
async fn get_confirmed_subscribers(
    db_pool: &PgPool
) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {
    let confirmed_subscribers = sqlx::query!(
        r#"
        SELECT email
        FROM subscriptions
        WHERE status = 'confirmed'
        "#,
    )
    .fetch_all(db_pool)
    .await?
    .into_iter()
    .map(|r| match SubscriberEmail::parse(r.email) {
        Ok(email) => Ok(ConfirmedSubscriber {email}),
        Err(error) => Err(anyhow::anyhow!(error))
    })
    .collect();

    Ok(confirmed_subscribers)
}