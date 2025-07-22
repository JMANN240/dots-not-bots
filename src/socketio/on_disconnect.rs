use std::sync::Arc;

use socketioxide::{
    SocketIo,
    extract::{SocketRef, State},
    socket::DisconnectReason,
};
use tracing::{info, instrument};

use super::SocketIoState;

#[instrument]
pub async fn on_disconnect(
    io: SocketIo,
    socket: SocketRef,
    State(state): State<Arc<SocketIoState>>,
    reason: DisconnectReason,
) {
    info!("Disconnected");

    if let Some(token) = state.socket_token.read().await.get(&socket.id) {
        info!(?token, "Found token");
        state.token_data.write().await.remove(token);
        info!(?token, "Removed token");
    }

    state.socket_token.write().await.remove(&socket.id);
    info!("Removed socket ID");

    io.emit("disconnected", &socket.id).await.unwrap();
    info!("Emitted disconnect");
}
