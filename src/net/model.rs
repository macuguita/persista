use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::identifier::Identifier;

#[derive(Deserialize)]
pub struct ChallengeRequest {
    pub id: String,
}

#[derive(Serialize)]
pub struct ChallengeResponse {
    pub token: String,
    pub expires_in: i32,
}

#[derive(Deserialize)]
pub struct AuthRequest {
    pub id: String,
    pub username: String,
    pub token: String,
}

#[derive(Serialize, Deserialize)]
pub struct SessionResponse {
    pub user_id: String,
    #[serde(rename = "session_token")]
    pub token: String,
    pub expires_at: String,
}

#[derive(Deserialize)]
pub struct MojangProfile {
    pub id: String,
    // pub name: String, // even if the field is there we don't need it
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Entitlements {
    pub values: Vec<String>,
}

impl Entitlements {
    pub fn empty() -> Self {
        Entitlements { values: Vec::new() }
    }

    pub fn validate(&self) -> Result<(), AppError> {
        for s in &self.values {
            if Identifier::try_parse(s).is_none() {
                return Err(AppError::BadRequest(format!("invalid identifier: {s}")));
            }
        }
        Ok(())
    }

    pub fn identifiers(&self) -> impl Iterator<Item = Identifier<'_>> {
        self.values.iter().filter_map(|s| Identifier::try_parse(s))
    }

    pub fn contains(&self, identifier: &Identifier) -> bool {
        self.identifiers().any(|id| &id == identifier)
    }
}