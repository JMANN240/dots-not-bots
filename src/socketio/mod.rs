use std::{collections::HashMap, sync::Arc};

use palette::{FromColor, Hsv, Srgb};
use serde::{Deserialize, Serialize};
use socketioxide::{
    SocketIo,
    extract::{Data, SocketRef, State},
    socket::Sid,
};
use sqlx::SqlitePool;
use tokio::sync::RwLock;
use tracing::{info, instrument};

mod on_disconnect;
mod on_position;

pub use on_disconnect::*;
pub use on_position::*;
use uuid::Uuid;

use crate::token_exists;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanData {
    pub id: Sid,
    pub position: Option<Position>,
    pub color: String,
}

#[derive(Debug)]
pub struct SocketIoState {
    pub socket_token: RwLock<HashMap<Sid, Uuid>>,
    pub token_data: RwLock<HashMap<Uuid, HumanData>>,
    pub pool: SqlitePool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocketIoAuth {
    pub token: Option<Uuid>,
}

#[instrument]
pub async fn on_connect(
    io: SocketIo,
    socket: SocketRef,
    Data(auth): Data<SocketIoAuth>,
    State(state): State<Arc<SocketIoState>>,
) {
    info!(?auth, "Connected");

    if let Some(token) = auth.token {
        if token_exists(&state.pool, &token).await.unwrap() {
            info!("Authenticated");

            let color = Srgb::from_color(Hsv::new(rand::random_range(0.0..360.0), 0.5, 1.0))
                .into_format::<u8>();

            info!(?color, "Color chosen");

            let new_data = HumanData {
                id: socket.id,
                position: None,
                color: format!("#{:02X}{:02X}{:02X}", color.red, color.green, color.blue),
            };

            info!(?new_data, "Data constructed");

            state.socket_token.write().await.insert(socket.id, token);
            info!(?state.socket_token, "Socket-token relationship written");

            state
                .token_data
                .write()
                .await
                .insert(token, new_data.clone());
            info!(?state.token_data, "Token-data relationship written");

            io.broadcast().emit("data", &new_data).await.unwrap();
            info!("New data broadcasted");

            socket.on("position", on_position);
            info!("Position handler added");

            socket.on_disconnect(on_disconnect);
            info!("Disconnect handler added");
        }
    }

    info!("Emitting existing data");
    for data in state.token_data.read().await.values() {
        socket.emit("data", data).unwrap();
        info!(?data, "Data emitted");
    }
    info!("All existing data emitted");
}
