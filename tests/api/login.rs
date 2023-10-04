use crate::helpers::{spawn_app, assert_is_redirect_to};

#[tokio::test]
async fn error_flash_message_is_set_on_failure() {
    let test_app = spawn_app().await;
    let login_body = serde_json::json!({
        "username": "random-username",
        "password": "random-password"
    });
    //Act 1: Try to login
    let response = test_app.post_login(&login_body).await;
    assert_is_redirect_to(&response, "/login");

    // Act 2: Follow the redirect
    let html_page = test_app.get_login_html().await;
    assert!(html_page.contains(r#"<p><i>Authentication failed</i></p>"#));
    
    //Act 3: Reload the login page
    let html_page = test_app.get_login_html().await;
    assert!(!html_page.contains(r#"<p><i>Authentication failed</i></p>"#));
}

#[tokio::test]
async fn redirect_to_admin_dashboard_after_login_success() {
    let test_app = spawn_app().await;
    let login_body = serde_json::json!({
        "username": &test_app.test_user.username,
        "password": &test_app.test_user.password
    });
    //Act 1: Try to login
    let response = test_app.post_login(&login_body).await;
    assert_is_redirect_to(&response, "/admin/dashboard");

    // Act 2: Follow the redirect
    let html_page = test_app.get_admin_dashboard_html().await;
    assert!(html_page.contains(&format!("Welcome {}", &test_app.test_user.username)));
}

#[tokio::test]
async fn you_must_be_logged_in_to_access_admin_dashboard() {
    let test_app = spawn_app().await;
    let response = test_app.get_admin_dashboard().await;

    assert_is_redirect_to(&response, "/login");
    
}
