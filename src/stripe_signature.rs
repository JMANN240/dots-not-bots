use axum::{extract::FromRequestParts, http::request::Parts};
use hmac::{Hmac, Mac};
use reqwest::StatusCode;
use sha2::Sha256;
use subtle::ConstantTimeEq;

#[derive(Debug)]
pub struct StripeSignature {
    pub timestamp: u64,
    pub v1: String,
}

impl StripeSignature {
    pub fn is_valid(&self, payload: &str, key: &[u8]) -> bool {
        let signed_payload = format!("{}.{}", self.timestamp, payload);

        let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC can take a key of any size");

        mac.update(signed_payload.as_bytes());

        let expected = mac.finalize().into_bytes();
        let expected_hex = hex::encode(expected);

        expected_hex.as_bytes().ct_eq(self.v1.as_bytes()).into()
    }
}

impl<S> FromRequestParts<S> for StripeSignature
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let sig_header = parts
            .headers
            .get("Stripe-Signature")
            .ok_or((
                StatusCode::BAD_REQUEST,
                "Missing Stripe-Signature header".to_string(),
            ))?
            .to_str()
            .map_err(|_| {
                (
                    StatusCode::BAD_REQUEST,
                    "Invalid header encoding".to_string(),
                )
            })?;

        let mut timestamp = None;
        let mut v1 = None;

        for entry in sig_header.split(',') {
            let mut kv = entry.splitn(2, '=');
            match (kv.next(), kv.next()) {
                (Some("t"), Some(val)) => {
                    timestamp =
                        Some(val.parse::<u64>().map_err(|_| {
                            (StatusCode::BAD_REQUEST, "Invalid timestamp".to_string())
                        })?);
                }
                (Some("v1"), Some(val)) => v1 = Some(val.to_string()),
                _ => {}
            }
        }

        match (timestamp, v1) {
            (Some(timestamp), Some(v1)) => Ok(Self { timestamp, v1 }),
            _ => Err((
                StatusCode::BAD_REQUEST,
                "Missing required fields in Stripe-Signature".into(),
            )),
        }
    }
}
