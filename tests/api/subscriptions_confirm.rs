use crate::helpers::spawn_app;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn confirmations_without_token_are_rejected_with_a_400() {
    let test_app = spawn_app().await;

    let response = reqwest::get(format!("{}/subscriptions/confirm", test_app.address))
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn the_link_returned_by_subscribe_returns_200_if_called() {
    let test_app = spawn_app().await;
    let body = "name=sang%20khuu&email=sangkhuudev%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&test_app.email_server)
        .await;
    test_app.post_subscriptions(body.into()).await;
    let email_request = &test_app.email_server.received_requests().await.unwrap()[0];
    let confirmation_link = test_app.get_confirmation_link(&email_request);

    assert_eq!(confirmation_link.html, confirmation_link.plain_text);

    let response = reqwest::get(confirmation_link.html).await.unwrap();
    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn clicking_on_the_confirmation_link_confirms_subscriber() {
    let test_app = spawn_app().await;
    let body = "name=sang%20khuu&email=sangkhuudev%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&test_app.email_server)
        .await;
    test_app.post_subscriptions(body.into()).await;
    let email_request = &test_app.email_server.received_requests().await.unwrap()[0];
    let confirmation_link = test_app.get_confirmation_link(&email_request);

    assert_eq!(confirmation_link.html, confirmation_link.plain_text);

    reqwest::get(confirmation_link.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions",)
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch saved subscriptions");
    assert_eq!(saved.email, "sangkhuudev@gmail.com");
    assert_eq!(saved.name, "sang khuu");
    assert_eq!(saved.status, "confirmed");
}
