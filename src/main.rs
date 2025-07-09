use std::{collections::HashMap, env, str::FromStr, sync::Arc};

use axum::{
    extract::State,
    routing::{get, post},
};
use axum_extra::extract::CookieJar;
use buy::buy;
use clap::Parser;
use dotenvy::dotenv;
use image::image;
use maud::{DOCTYPE, Markup, html};
use register::register;
use serde::{Deserialize, Serialize};
use set_token::set_token;
use socketio::on_connect;
use socketioxide::{socket::Sid, SocketIo};
use sqlx::SqlitePool;
use tokio::sync::RwLock;
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::FmtSubscriber;
use uuid::Uuid;

mod buy;
mod image;
mod register;
mod set_token;
mod socketio;
mod stripe_signature;

#[derive(Parser)]
struct Cli {
    port: u16,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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

#[derive(Clone)]
struct AppState {
    pub socket_token: Arc<RwLock<HashMap<Sid, Uuid>>>,
    pub token_data: Arc<RwLock<HashMap<Uuid, HumanData>>>,
    pub pool: SqlitePool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing::subscriber::set_global_default(FmtSubscriber::default())?;
    let cli = Cli::parse();
    dotenv()?;

    let pool = SqlitePool::connect(
        &env::var("DATABASE_URL").expect("DATABASE_URL environment variable not set"),
    )
    .await
    .unwrap();

    let state = AppState {
        socket_token: Arc::new(RwLock::new(HashMap::new())),
        token_data: Arc::new(RwLock::new(HashMap::new())),
        pool,
    };

    let (layer, io) = SocketIo::builder()
        .with_state(state.clone())
        .build_layer();

    io.ns("/", on_connect);

    let app = axum::Router::new()
        .route("/", get(root))
        .route("/buy", get(buy))
        .route("/set", get(set_token))
        .route("/image", get(image))
        .route("/register", post(register))
        .fallback_service(ServeDir::new("static"))
        .with_state(state)
        .layer(layer);

    info!("Starting server");

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", cli.port)).await.unwrap();
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
                (get_meta_tags())
                link rel="stylesheet" href="/styles.css";
            }
            body .roboto-100 {
                p { "Welcome to the last real website." }
                @if read_only {
                    p {
                        "Come watch, or "
                        a href="/buy" {
                            "join us"
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

pub fn get_meta_tags() -> Markup {


    let title = "DotsNotBots Club";
    let description = "Come be human.";

    html! {
        title { (title) }
        meta name="description" content=(description);

        meta property="og:url" content="https://dotsnotbots.club/";
        meta property="og:type" content="website";
        meta property="og:title" content=(title);
        meta property="og:description" content=(description);

        meta name="twitter:card" content="summary_large_image";
        meta property="twitter:domain" content="dotsnotbots.club";
        meta property="twitter:url" content="https://dotsnotbots.club/";
        meta name="twitter:title" content=(title);
        meta name="twitter:description" content=(description);
    }
}
