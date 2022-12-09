use app::{sycamore::render_to_string, AppProps, ClientMessage, ServerMessage};
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, get_service},
    Router,
};
use lazy_static::lazy_static;
use serde::Serialize;
use std::{
    net::SocketAddr,
    sync::atomic::{AtomicI64, Ordering},
};
use tokio::sync::Mutex;
use tower_http::services::ServeFile;

lazy_static! {
    static ref TEMPLATE: String = include_str!("../index.html")
        .replace(
            "%app.script%",
            &format!(
                "<script type=\"module\">{}</script>",
                include_str!("../dist/wasm/init.min.js")
            ),
        )
        .replace(
            "%app.style%",
            &format!("<style>{}</style>", include_str!("../dist/style.css")),
        );
    static ref WEBSOCKETS: Mutex<Vec<WebSocket>> = Mutex::new(Vec::new());
}

static COUNT: AtomicI64 = AtomicI64::new(0);

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(handler))
        .route(
            "/client_bg.wasm",
            get_service(ServeFile::new("dist/wasm/client_bg.wasm")).handle_error(handle_error),
        )
        .route("/ws", get(handle_ws));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    tokio::spawn(async {
        let mut previous_count = COUNT.load(Ordering::SeqCst);
        loop {
            let current = COUNT.load(Ordering::SeqCst);
            if previous_count != current {
                previous_count = current;
                let mut websockets = WEBSOCKETS.lock().await;
                let data = postcard::to_stdvec(&ServerMessage::Set(current))
                    .expect("Failed to serialize server message");
                for websocket in websockets.iter_mut() {
                    let _ = websocket.send(Message::Binary(data.clone())).await;
                }
            }
        }
    });

    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler() -> Html<String> {
    let props = AppProps {
        count: COUNT.load(Ordering::SeqCst),
    };

    Html(
        TEMPLATE
            .replace(
                "%app.props%",
                &base64::encode(postcard::to_stdvec(&props).unwrap_or_else(|_| Vec::new())),
            )
            .replace("%app.root%", &render_to_string(|cx| app::App(cx, props))),
    )
}

async fn handle_ws(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(|mut socket: WebSocket| async move {
        while let Some(message) = socket.recv().await {
            let Ok(message) = message else {
                // client disconnected
                return;
            };

            let Message::Binary(message) = message else {
                continue;
            };

            let Ok(message) = postcard::from_bytes(&message) else {
                continue;
            };

            match message {
                ClientMessage::Increment => {
                    COUNT.fetch_add(1, Ordering::SeqCst);
                }
                ClientMessage::Decrement => {
                    COUNT.fetch_add(-1, Ordering::SeqCst);
                }
            }
        }
    })
}

async fn handle_error(_err: std::io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}
