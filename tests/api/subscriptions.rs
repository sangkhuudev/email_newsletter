use crate::helpers::spawn_app;


#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form() {
    let test_app = spawn_app().await;
    let body = "name=sang%20khuu&email=sangkhuudev%40gmail.com";
    let response = test_app.post_subscriptions(body.into()).await;
        
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch saved subscriptions");
    assert_eq!(saved.email, "sangkhuudev@gmail.com");
    assert_eq!(saved.name, "sang khuu");
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
