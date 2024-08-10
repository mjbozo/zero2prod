//! src/routes/admin/newsletters/get.rs

use actix_web::{http::header::ContentType, HttpResponse};
use actix_web_flash_messages::IncomingFlashMessages;
use std::fmt::Write;

pub async fn publish_newsletter_form(
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
<html>
<head>
    <title>Send Newsletter Issue</title>
    <meta http-equiv="content-type" content="text/html; charset=utf-8">
</head>
<body>
    {msg_html}
    <form action="/admin/newsletters" method="post">
      <label>
        Title
        <input type="text" placeholder="Enter newsletter issue title" name="title">
        <br>
      </label>
      <br>
      <label>
        Text Content
        <br>
        <textarea placeholder="Enter text content" name="text" rows="20" cols="50"></textarea>
        <br>
      </label>
      <br>
      <label>
        Html Content
        <br>
        <textarea placeholder="Enter html content" name="html" rows="20" cols="50"></textarea>
        <br>
      </label>
      <br>

      <button type="submit">Send</button>
    </form>

    <p><a href="/admin/dashboard">‹ Back</a></p>
</body>
</html>"#
        )));
}
