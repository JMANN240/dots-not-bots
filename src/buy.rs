use std::env;

use axum::response::Redirect;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct SessionResponse {
    url: String,
}

pub async fn buy() -> Redirect {
    let client = reqwest::Client::new();

    let uuid = Uuid::new_v4();

    let host = env::var("HOST").expect("HOST environment variable is not set");

    let success_url = format!("{}/set?token={}", host, uuid.as_hyphenated());

    let params = [
        ("mode", "payment"),
        ("success_url", &success_url),
        (
            "payment_intent_data[description]",
            &format!("Your Human Token: {}", uuid),
        ),
        (
            "line_items[0][price]",
            &env::var("MEMBERSHIP_PRICE_ID")
                .expect("MEMBERSHIP_PRICE_ID environment variable is not set"),
        ),
        ("line_items[0][quantity]", "1"),
        ("metadata[human_token]", &uuid.as_hyphenated().to_string()),
    ];

    let session_response = client
        .post("https://api.stripe.com/v1/checkout/sessions")
        .basic_auth(
            env::var("STRIPE_SECRET_KEY")
                .expect("STRIPE_SECRET_KEY environment variable is not set"),
            None::<&str>,
        )
        .form(&params)
        .send()
        .await
        .unwrap();

    let session_json = session_response.json::<SessionResponse>().await.unwrap();

    Redirect::to(&session_json.url)
}
