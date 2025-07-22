use std::sync::Arc;

use socketioxide::{
    SocketIo,
    extract::{Data, SocketRef, State},
};
use tracing::{info, instrument};

use super::{Position, SocketIoState};

#[instrument]
pub async fn on_position(
    io: SocketIo,
    socket: SocketRef,
    Data(position): Data<Position>,
    State(state): State<Arc<SocketIoState>>,
) {
    info!("Received position");

    if let Some(token) = state.socket_token.read().await.get(&socket.id) {
        info!(?token, "Found token");

        let mut token_data = state.token_data.write().await;
        info!("Acquired token-data write lock");

        let data = token_data.get_mut(token).unwrap();
        info!(?token, ?data, "Got token data");

        data.position = Some(position);
        info!(?data, "Set new position");

        io.broadcast().emit("data", &data).await.unwrap();
        info!("New data broadcasted");
    }
}
