use actix_web::http::header::ContentType;
use actix_web::HttpResponse;
use actix_web_flash_messages::{IncomingFlashMessages};
use std::fmt::Write;
//----------------------------------------------------------------
#[derive(serde::Deserialize)]
pub struct QueryParams {
    pub error: String,
    pub tag: String,
}

//----------------------------------------------------------------
pub async fn login_form(flash_messages: IncomingFlashMessages) -> HttpResponse {
    let mut error_html = String::new();
    // Display all messages levels, not just errors!
    for m in flash_messages.iter() {
        writeln!(error_html, "<p><i>{}</i></p>", m.content()).unwrap();
    }

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"
            <!DOCTYPE html>
<html lang="en">
<head>
    <meta http-equiv="content-type" content="text/html; charset=utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Login</title>
</head>
<body>
    {error_html}
    <form action="login" method="POST">
        <label>Username
            <input 
                type="text"
                placeholder="Enter username"
                name="username"
            >
        </label>
        <label>Password
            <input 
                type="password"
                placeholder="Enter password"
                name="password"
            >
        </label>
        <button type="submit">Login</button>
    </form>
</body>
</html>
            "#,
        ))
}
