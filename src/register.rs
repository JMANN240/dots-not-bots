use std::env;

use axum::extract::State;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{AppState, stripe_signature::StripeSignature};

#[derive(Serialize, Deserialize, Debug)]
struct Event {
    data: EventData,
}

#[derive(Serialize, Deserialize, Debug)]
struct EventData {
    object: EventDataObject,
}

#[derive(Serialize, Deserialize, Debug)]
struct EventDataObject {
    metadata: EventDataObjectMetadata,
}

#[derive(Serialize, Deserialize, Debug)]
struct EventDataObjectMetadata {
    human_token: Uuid,
}

pub async fn register(
    stripe_signature: StripeSignature,
    State(state): State<AppState>,
    body: String,
) {
    let key = env::var("REGISTER_SIGNING_KEY")
        .expect("REGISTER_SIGNING_KEY environment variable is not set");

    // TODO: timestamp verification?

    if stripe_signature.is_valid(&body, key.as_bytes()) {
        let event = serde_json::from_str::<Event>(&body).unwrap();
        let human_token = event.data.object.metadata.human_token;
        let human_token_string = human_token.as_hyphenated().to_string();

        sqlx::query!("INSERT INTO human_tokens VALUES (?)", human_token_string)
            .execute(&state.pool)
            .await
            .unwrap();
    }
}
