//! src/email_client.rs

use crate::domain::SubscriberEmail;
use reqwest::Client;
use secrecy::{ExposeSecret, Secret};

pub struct EmailClient {
    base_url: String,
    http_client: Client,
    sender: SubscriberEmail,
    authorisation_token: Secret<String>,
}

impl EmailClient {
    pub fn new(
        base_url: String,
        sender: SubscriberEmail,
        authorisation_token: Secret<String>,
        timeout: std::time::Duration,
    ) -> Self {
        return Self {
            http_client: Client::builder().timeout(timeout).build().unwrap(),
            base_url,
            sender,
            authorisation_token,
        };
    }
    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}/v3/mail/send", self.base_url);
        let request_body = SendEmailRequest {
            from: Email {
                email: self.sender.as_ref(),
            },
            // personalizations here instead of `to`, as SendGrid has different schema to Postmark
            personalizations: vec![Personalization {
                to: vec![Email {
                    email: recipient.as_ref(),
                }],
            }],
            subject,
            content: vec![
                EmailContent {
                    content_type: EmailContentType::Html,
                    value: html_content,
                },
                EmailContent {
                    content_type: EmailContentType::Text,
                    value: text_content,
                },
            ],
        };
        self.http_client
            .post(&url)
            .header(
                "Authorization",
                format!("Bearer {}", self.authorisation_token.expose_secret()),
            )
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?;
        return Ok(());
    }
}

// Structure differs from the book here since Postmark won't allow accounts without private email domain
// Using SendGrid instead because it had the best docs
#[derive(serde::Serialize)]
struct Email<'a> {
    email: &'a str,
}

#[derive(serde::Serialize)]
struct Personalization<'a> {
    to: Vec<Email<'a>>,
}

#[derive(serde::Serialize)]
enum EmailContentType {
    #[serde(rename = "text/plain")]
    Text,
    #[serde(rename = "text/html")]
    Html,
}

#[derive(serde::Serialize)]
struct EmailContent<'a> {
    value: &'a str,
    #[serde(rename = "type")]
    content_type: EmailContentType,
}

#[derive(serde::Serialize)]
struct SendEmailRequest<'a> {
    from: Email<'a>,
    personalizations: Vec<Personalization<'a>>,
    subject: &'a str,
    content: Vec<EmailContent<'a>>,
}
// end changes for using SendGrid instead of Postmark

#[cfg(test)]
mod tests {
    use claims::{assert_err, assert_ok};
    use fake::{
        faker::{
            internet::en::SafeEmail,
            lorem::en::{Paragraph, Sentence},
        },
        Fake, Faker,
    };
    use secrecy::Secret;
    use wiremock::{matchers::any, Request};
    use wiremock::{
        matchers::{header, header_exists, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use crate::{domain::SubscriberEmail, email_client::EmailClient};

    struct SendEmailBodyMatcher;

    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &Request) -> bool {
            let result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);
            if let Ok(body) = result {
                return body.get("personalizations").is_some()
                    && body.get("personalizations").unwrap().is_array()
                    && body.get("personalizations").unwrap()[0].get("to").is_some()
                    && body.get("personalizations").unwrap()[0]
                        .get("to")
                        .unwrap()
                        .is_array()
                    && body.get("personalizations").unwrap()[0].get("to").unwrap()[0]
                        .get("email")
                        .is_some()
                    && body.get("from").is_some()
                    && body.get("from").unwrap().get("email").is_some()
                    && body.get("subject").is_some()
                    && body.get("content").is_some()
                    && body.get("content").unwrap().is_array()
                    && body.get("content").unwrap()[0].is_object()
                    && body.get("content").unwrap()[0].get("type").is_some()
                    && body.get("content").unwrap()[0].get("value").is_some();
            } else {
                return false;
            }
        }
    }

    /// Generate random email subject
    fn subject() -> String {
        return Sentence(1..2).fake();
    }

    /// Generate random email content
    fn content() -> String {
        return Paragraph(1..10).fake();
    }

    /// Generate random subscriber email
    fn email() -> SubscriberEmail {
        return SubscriberEmail::parse(SafeEmail().fake()).unwrap();
    }

    /// Get a test instance of `EmailClient`
    fn email_client(base_url: String) -> EmailClient {
        return EmailClient::new(
            base_url,
            email(),
            Secret::new(Faker.fake()),
            std::time::Duration::from_millis(200),
        );
    }

    #[tokio::test]
    async fn send_email_sends_the_expected_request() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(header_exists("Authorization"))
            .and(header("Content-Type", "application/json"))
            .and(path("/v3/mail/send"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let _ = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;
    }

    #[tokio::test]
    async fn send_email_succeeds_if_server_returns_200() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let outcome = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        assert_ok!(outcome);
    }

    #[tokio::test]
    async fn send_email_fails_if_server_returns_500() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        let outcome = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        assert_err!(outcome);
    }

    #[tokio::test]
    async fn send_email_times_out_if_server_takes_too_long() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(180)))
            .expect(1)
            .mount(&mock_server)
            .await;

        let outcome = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        assert_err!(outcome);
    }
}
