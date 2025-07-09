use std::{collections::HashMap, env, str::FromStr, sync::Arc};

use axum::{
    extract::State,
    routing::{get, post},
};
use axum_extra::extract::CookieJar;
use buy::buy;
use dotenvy::dotenv;
use maud::{DOCTYPE, Markup, html};
use register::register;
use set_token::set_token;
use socketio::{SocketIoState, on_connect};
use socketioxide::SocketIo;
use sqlx::SqlitePool;
use tokio::sync::RwLock;
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::FmtSubscriber;
use uuid::Uuid;

mod buy;
mod register;
mod set_token;
mod socketio;
mod stripe_signature;

#[derive(Clone)]
struct AppState {
    pool: SqlitePool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing::subscriber::set_global_default(FmtSubscriber::default())?;
    dotenv()?;

    let pool = SqlitePool::connect(
        &env::var("DATABASE_URL").expect("DATABASE_URL environment variable not set"),
    )
    .await
    .unwrap();

    let (layer, io) = SocketIo::builder()
        .with_state(Arc::new(SocketIoState {
            socket_token: RwLock::new(HashMap::new()),
            token_data: RwLock::new(HashMap::new()),
            pool: pool.clone(),
        }))
        .build_layer();

    io.ns("/", on_connect);

    let app_state = AppState { pool };

    let app = axum::Router::new()
        .route("/", get(root))
        .route("/buy", get(buy))
        .route("/set", get(set_token))
        .route("/register", post(register))
        .fallback_service(ServeDir::new("static"))
        .with_state(app_state)
        .layer(layer);

    info!("Starting server");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

pub async fn token_exists(pool: &SqlitePool, token: &Uuid) -> sqlx::Result<bool> {
    let token_string = token.as_hyphenated().to_string();

    Ok(
        sqlx::query!("SELECT * FROM human_tokens WHERE token = ?", token_string)
            .fetch_optional(pool)
            .await?
            .is_some(),
    )
}

async fn root(State(state): State<AppState>, jar: CookieJar) -> Markup {
    let token_cookie = jar.get("token");

    let maybe_token_string = token_cookie.map(|token| token.value_trimmed().to_string());

    let read_only = match &maybe_token_string {
        Some(token_string) => match Uuid::from_str(token_string) {
            Ok(token) => !token_exists(&state.pool, &token).await.unwrap(),
            Err(_) => true,
        },
        None => true,
    };

    html! {
        (DOCTYPE)
        html {
            head {
                link rel="stylesheet" href="/styles.css";
            }
            body .roboto-100 {
                p { "Welcome to the last real website." }
                @if read_only {
                    p {
                        "Come watch, or "
                        a href="/buy" {
                            "join in"
                        }
                        " for $1.00."
                    }
                } @else {
                    p { "Join us. Click to place yourself." }
                }
                canvas width="512" height="512" {}
                p { "Every dot is a human with us now." }
                p { "When you leave, so does your dot." }

                @if let Some(token_string) = maybe_token_string {
                    span id="token-span" {
                        "Token: "
                        span {
                            (token_string)
                        }
                    }
                }
                script src="https://cdn.socket.io/4.8.1/socket.io.min.js" integrity="sha384-mkQ3/7FUtcGyoppY6bz/PORYoGqOl7/aSUMn2ymDOJcapfS6PHqxhRTMh1RR0Q6+" crossorigin="anonymous" {}
                script src="/script.js" {}
            }
        }
    }
}
