use socketioxide::{
    SocketIo,
    extract::{SocketRef, State},
    socket::DisconnectReason,
};
use tracing::info;

use crate::AppState;

pub async fn on_disconnect(
    io: SocketIo,
    socket: SocketRef,
    State(state): State<AppState>,
    reason: DisconnectReason,
) {
    info!("Socket {} disconnected: {}", socket.id, reason);

    if let Some(token) = state.socket_token.read().await.get(&socket.id) {
        state.token_data.write().await.remove(token);
    }

    state.socket_token.write().await.remove(&socket.id);

    io.emit("disconnected", &socket.id).await.unwrap();
}
