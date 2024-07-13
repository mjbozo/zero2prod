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
    ) -> Self {
        return Self {
            http_client: Client::new(),
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
                email: self.sender.as_ref().to_owned(),
            },
            // personalizations here instead of `to`, as SendGrid has different schema to Postmark
            personalizations: vec![Personalization {
                to: vec![Email {
                    email: recipient.as_ref().to_owned(),
                }],
            }],
            subject: subject.to_owned(),
            content: vec![
                EmailContent {
                    content_type: EmailContentType::Html,
                    value: html_content.to_owned(),
                },
                EmailContent {
                    content_type: EmailContentType::Text,
                    value: text_content.to_owned(),
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
            .await?;
        return Ok(());
    }
}

// Structure differs from the book here since Postmark won't allow accounts without private email domain
// Using SendGrid instead because it had the best docs
#[derive(serde::Serialize)]
struct Email {
    email: String,
}

#[derive(serde::Serialize)]
struct Personalization {
    to: Vec<Email>,
}

#[derive(serde::Serialize)]
enum EmailContentType {
    #[serde(rename = "text/plain")]
    Text,
    #[serde(rename = "text/html")]
    Html,
}

#[derive(serde::Serialize)]
struct EmailContent {
    value: String,
    #[serde(rename = "type")]
    content_type: EmailContentType,
}

#[derive(serde::Serialize)]
struct SendEmailRequest {
    from: Email,
    personalizations: Vec<Personalization>,
    subject: String,
    content: Vec<EmailContent>,
}
// end changes for using SendGrid instead of Postmark

#[cfg(test)]
mod tests {
    use fake::{
        faker::{
            internet::en::SafeEmail,
            lorem::en::{Paragraph, Sentence},
        },
        Fake, Faker,
    };
    use secrecy::Secret;
    use wiremock::Request;
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

    #[tokio::test]
    async fn send_email_sends_the_expected_request() {
        let mock_server = MockServer::start().await;
        let sender = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let email_client = EmailClient::new(mock_server.uri(), sender, Secret::new(Faker.fake()));

        Mock::given(header_exists("Authorization"))
            .and(header("Content-Type", "application/json"))
            .and(path("/v3/mail/send"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        let _ = email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;
    }
}
