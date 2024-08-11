//! src/idempotency/key.rs

#[derive(Debug)]
pub struct IdempotencyKey(String);

impl TryFrom<String> for IdempotencyKey {
    type Error = anyhow::Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        if s.is_empty() {
            anyhow::bail!("The idempotency key cannot be empty");
        }

        let max_length = 50;
        if s.len() >= max_length {
            anyhow::bail!("The idempotency key must be shorter than {max_length} characters");
        }

        return Ok(Self(s));
    }
}

impl From<IdempotencyKey> for String {
    fn from(k: IdempotencyKey) -> Self {
        return k.0;
    }
}

impl AsRef<str> for IdempotencyKey {
    fn as_ref(&self) -> &str {
        return &self.0;
    }
}
