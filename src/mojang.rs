use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use reqwest::Client;

use crate::error::AppError;
use crate::model::MojangProfile;

#[derive(Clone)]
pub struct MojangAuth {
    client: Client,
    challenges: Arc<Mutex<HashMap<String, (String, u128)>>>,
}

impl MojangAuth {
    pub fn new() -> Self {
        MojangAuth {
            client: Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .expect("failed to build HTTP client"),
            challenges: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn generate_challenge(&self) -> String {
        use std::fmt::Write;
        let mut bytes = [0u8; 20];

        getrandom::fill(&mut bytes).expect("failed to generate random bytes");
        bytes.iter().fold(String::with_capacity(40), |mut s, b| {
            write!(s, "{b:02x}").unwrap();
            s
        })
    }

    pub fn store_challenge(&self, player_id: &str, challenge: String) {
        let expires_at = now_millis() + 30_000;
        let mut map = self.challenges.lock().expect("challenge lock poisoned");
        Self::purge_if_expired(&mut map);
        map.insert(player_id.to_string(), (challenge, expires_at));
    }

    pub fn consume_challenge(&self, player_id: &str) -> Option<String> {
        let mut map = self.challenges.lock().expect("challenge lock poisoned");
        let (challenge, expires_at) = map.remove(player_id)?;
        if now_millis() > expires_at {
            None
        } else {
            Some(challenge)
        }
    }

    pub async fn verify_with_mojang(
        &self,
        username: &str,
        challenge: &str,
    ) -> Result<Option<MojangProfile>, AppError> {
        let response = self
            .client
            .get("https://sessionserver.mojang.com/session/minecraft/hasJoined")
            .query(&[("username", username), ("serverId", challenge)])
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::NO_CONTENT {
            return Ok(None);
        }
        if !response.status().is_success() {
            return Ok(None);
        }

        let profile = response.json::<MojangProfile>().await?;

        Ok(Some(profile))
    }

    fn purge_if_expired(map: &mut HashMap<String, (String, u128)>) {
        let now = now_millis();
        map.retain(|_, (_, expires_at)| *expires_at > now);
    }
}

fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is before 1970")
        .as_millis()
}
