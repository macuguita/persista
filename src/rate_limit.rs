use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    body::Body,
    extract::Request,
    http::{HeaderValue, Response, StatusCode},
    middleware::Next,
};

#[derive(Clone)]
struct Bucket {
    tokens: u32,
    refills_at: u64,
}

#[derive(Clone)]
pub struct RateLimiter {
    buckets: Arc<Mutex<HashMap<String, Bucket>>>,
    limit: u32,
    refill_period_secs: u64,
}

impl RateLimiter {
    pub fn new(limit: u32, refill_period_secs: u64) -> Self {
        RateLimiter {
            buckets: Arc::new(Mutex::new(HashMap::new())),
            limit,
            refill_period_secs,
        }
    }

    fn check(&self, key: &str) -> Result<(u32, u64), u64> {
        let now = now_secs();
        let mut map = self.buckets.lock().expect("rate limiter lock poisoned");

        let bucket = map.entry(key.to_string()).or_insert_with(|| Bucket {
            tokens: self.limit,
            refills_at: now + self.refill_period_secs,
        });

        if now >= bucket.refills_at {
            bucket.tokens = self.limit;
            bucket.refills_at = now + self.refill_period_secs;
        }

        if bucket.tokens == 0 {
            return Err(bucket.refills_at);
        }

        bucket.tokens -= 1;
        Ok((bucket.tokens, bucket.refills_at))
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is before 1970")
        .as_secs()
}

fn extract_key(req: &Request<Body>) -> String {
    req.headers()
        .get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

pub async fn rate_limit_middleware(
    limiter: Arc<RateLimiter>,
    req: Request<Body>,
    next: Next,
) -> Response<Body> {
    let key = extract_key(&req);

    match limiter.check(&key) {
        Ok((remaining, reset_at)) => {
            let mut response = next.run(req).await;
            let headers = response.headers_mut();
            headers.insert("X-RateLimit-Limit", HeaderValue::from(limiter.limit));
            headers.insert("X-RateLimit-Remaining", HeaderValue::from(remaining));
            headers.insert("X-RateLimit-Reset", HeaderValue::from(reset_at));
            response
        }
        Err(reset_at) => {
            let body = serde_json::json!({ "error": "too many requests" }).to_string();
            Response::builder()
                .status(StatusCode::TOO_MANY_REQUESTS)
                .header("Content-Type", "application/json")
                .header("X-RateLimit-Limit", limiter.limit)
                .header("X-RateLimit-Remaining", 0u32)
                .header("X-RateLimit-Reset", reset_at)
                .body(Body::from(body))
                .unwrap()
        }
    }
}
