use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    exp: usize,
}

impl Claims {
    pub fn with_email(email: &str) -> Self {
        Self {
            sub: email.to_owned(),
            exp: (Utc::now() + Duration::hours(24)).timestamp() as usize,
        }
    }
}