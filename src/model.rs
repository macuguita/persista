use serde::{Deserialize, Serialize};

use crate::error::AppError;

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
    pub session_token: String,
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
            if crate::identifier::Identifier::try_parse(s).is_none() {
                return Err(AppError::BadRequest(format!("invalid identifier: {s}")));
            }
        }
        Ok(())
    }

    pub fn to_identifiers(&self) -> Vec<crate::identifier::Identifier> {
        self.values
            .iter()
            .filter_map(|s| crate::identifier::Identifier::try_parse(s))
            .collect()
    }

    pub fn contains(&self, identifier: &crate::identifier::Identifier) -> bool {
        self.to_identifiers().contains(identifier)
    }
}
