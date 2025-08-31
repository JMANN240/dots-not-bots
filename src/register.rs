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
    line_items: EventDataObjectLineItems,
    metadata: EventDataObjectMetadata,
}

#[derive(Serialize, Deserialize, Debug)]
struct EventDataObjectLineItems {
    data: Vec<LineItem>,
}

#[derive(Serialize, Deserialize, Debug)]
struct LineItem {
    price: Price,
}

#[derive(Serialize, Deserialize, Debug)]
struct Price {
    id: String,
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

    let price_id = env::var("MEMBERSHIP_PRICE_ID")
        .expect("MEMBERSHIP_PRICE_ID environment variable is not set");

    // TODO: timestamp verification?

    if stripe_signature.is_valid(&body, key.as_bytes()) {
        let event = serde_json::from_str::<Event>(&body).unwrap();

        if event
            .data
            .object
            .line_items
            .data
            .first()
            .is_some_and(|line_item| line_item.price.id == price_id)
        {
            let human_token = event.data.object.metadata.human_token;
            let human_token_string = human_token.as_hyphenated().to_string();

            sqlx::query!("INSERT INTO human_tokens VALUES (?)", human_token_string)
                .execute(&state.pool)
                .await
                .unwrap();
        }
    }
}
