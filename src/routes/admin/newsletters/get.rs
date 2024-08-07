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
      </label>
      <label>
        Text Content
        <input type="text" placeholder="Enter text content" name="text">
      </label>
      <label>
        Html Content
        <input type="text" placeholder="Enter html content" name="html">
      </label>

      <button type="submit">Send</button>
    </form>
</body>
</html>"#
        )));
}
