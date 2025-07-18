use std::sync::Arc;

use socketioxide::{
    SocketIo,
    extract::{Data, SocketRef, State},
};
use tracing::info;

use super::{Position, SocketIoState};

pub async fn on_position(
    io: SocketIo,
    socket: SocketRef,
    Data(position): Data<Position>,
    State(state): State<Arc<SocketIoState>>,
) {
    info!("Socket {} position {:?}", socket.id, position);

    if let Some(token) = state.socket_token.read().await.get(&socket.id) {
        let mut socket_data = state.token_data.write().await;
        let data = socket_data.get_mut(token).unwrap();
        data.position = Some(position);
        io.broadcast().emit("data", &data).await.unwrap();
    }
}
