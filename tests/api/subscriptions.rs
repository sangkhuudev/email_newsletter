use crate::helpers::spawn_app;
use wiremock::matchers::{path, method};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form() {
    let test_app = spawn_app().await;
    let body = "name=sang%20khuu&email=sangkhuudev%40gmail.com";
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    let response = test_app.post_subscriptions(body.into()).await;
        
    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_persists_to_new_subscriber() {
    let test_app = spawn_app().await;
    let body = "name=sang%20khuu&email=sangkhuudev%40gmail.com";
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    test_app.post_subscriptions(body.into()).await;
    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions",)
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch saved subscriptions");
    assert_eq!(saved.email, "sangkhuudev@gmail.com");
    assert_eq!(saved.name, "sang khuu");
    assert_eq!(saved.status, "pending_confirmation");
}
#[tokio::test]
async fn subscribe_returns_a_400_for_missing_data() {
    let test_app = spawn_app().await;
    let test_cases = vec![
        ("name=sang%20khuu", "missing the email"),
        ("email=sanghuudev%40gmail.com", "missing the nam"),
        ("", "missing both name and email")
    ];
    
    for (invalid_body, error_message) in test_cases {
        let response = test_app.post_subscriptions(invalid_body.into()).await;
        assert_eq!(400, response.status().as_u16(),
            "The API did not fail with 400 Bad request when the payload was {}",
            error_message
        )
    }
}

#[tokio::test]
async fn subscribe_returns_a_400_when_fields_are_present_but_invalid() {
    let test_app = spawn_app().await;
    let test_cases = vec![
        ("name=&email=sangkhuu%40gmail.com", "empty name"),
        ("name=sang&email=", "empty email"),
        ("name=sang&email=not-an-email", "invalid email"),
    ];
    
    for (invalid_body, error_message) in test_cases {
        let response = test_app.post_subscriptions(invalid_body.into()).await;

        assert_eq!(400, response.status().as_u16(),
            "The API did not return a 200 OK when the payload was {}",
            error_message
        )
    }
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_with_valid_data() {
    let test_app = spawn_app().await;
    let body = "name=sang%20khuu&email=sangkhuudev%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;
    test_app.post_subscriptions(body.into()).await;

}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_with_a_link() {
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
}

#[tokio::test]
async fn subscribe_fails_if_there_is_a_fatal_database_error() {
    let test_app = spawn_app().await;
    let body = "name=sang%20khuu&email=sangkhuudev%40gmail.com";
    let response = test_app.post_subscriptions(body.into()).await;
    sqlx::query!("ALTER TABLE subscription_tokens DROP COLUMN subscription_token;",)
        .execute(&test_app.db_pool)
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 500);
}