use std::sync::Arc;

use socketioxide::{
    SocketIo,
    extract::{SocketRef, State},
    socket::DisconnectReason,
};
use tracing::{info, warn};

use super::SocketIoState;

pub async fn on_disconnect(
    io: SocketIo,
    socket: SocketRef,
    State(state): State<Arc<SocketIoState>>,
    reason: DisconnectReason,
) {
    info!("Socket {} disconnected: {}", socket.id, reason);

    if let Some(token) = state.socket_token.read().await.get(&socket.id) {
        state.token_data.write().await.remove(token);
    }

    state.socket_token.write().await.remove(&socket.id);

    for socket in io.sockets() {
        if let Err(error) = socket.emit("disconnected", &socket.id) {
            warn!("Failed to emit to socket {}, dropping: {}", socket.id, error);
            socket.disconnect().unwrap();
        }
    }
}
