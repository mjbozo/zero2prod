//! src/routes/admin/password/get.rs

use actix_web::http::header::ContentType;
use actix_web::HttpResponse;
use actix_web_flash_messages::IncomingFlashMessages;
use std::fmt::Write;

pub async fn change_password_form(
    flash_messages: IncomingFlashMessages,
) -> Result<HttpResponse, actix_web::Error> {
    let mut msg_html = String::new();
    for m in flash_messages.iter() {
        writeln!(msg_html, "<p><i>{}</i></p>", m.content()).unwrap();
    }

    return Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <title>Change Password</title>
    <meta http-equiv="content-type" content="text/html; charset=utf-8">
</head>
<body>
    {msg_html}
    <form action="/admin/password" method="post">
        <label>
            Current password
            <input
                type="password"
                placeholder="Enter current password"
                name="current_password"
            >
        </label>
        <br>
        <label>
            New password
            <input
                type="password"
                placeholder="Enter new password"
                name="new_password"
            >
        </label>
        <br>
        <label>
            Confirm new password
            <input
                type="password"
                placeholder="Type new password again"
                name="new_password_check"
            >
        </label>
    </form>
    <p><a href="/admin/dashboard">‹ Back</a></p>
</body>
</html>"#
        )));
}
