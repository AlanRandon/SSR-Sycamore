use app::{sycamore::render_to_string, AppProps};
use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade},
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

            dbg!(message);

            // if socket.send(msg).await.is_err() {
            //     return;
            // }
        }
    })
}

async fn handle_error(_err: std::io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}
