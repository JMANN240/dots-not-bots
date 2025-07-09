use std::io::Cursor;

use axum::{extract::State, http::HeaderMap, response::IntoResponse};
use image::{Rgb, RgbImage};

use crate::AppState;

pub async fn image(State(state): State<AppState>) -> impl IntoResponse {
    let mut image = RgbImage::new(512, 512);

    for data in state.token_data.read().await.values() {
        if let Some(position) = data.position {
            image.put_pixel(position.x as u32, position.y as u32, Rgb([255,0,0]));
        }
    }

    let image_bytes = Vec::new();
    let mut image_bytes_cursor = Cursor::new(image_bytes);
    image.write_to(&mut image_bytes_cursor, image::ImageFormat::Png)
        .unwrap();

    let mut headers = HeaderMap::new();

    headers.insert(axum::http::header::CONTENT_DISPOSITION, "inline".parse().unwrap());
    headers.insert(axum::http::header::CONTENT_TYPE, "image/png".parse().unwrap());

    (headers, image_bytes_cursor.into_inner())
}
