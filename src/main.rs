use app::sycamore::render_to_string;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, get_service},
    Router,
};
use std::net::SocketAddr;
use tower_http::services::ServeFile;

const TEMPLATE: &str = include_str!("../index.html");

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(handler)).route(
        "/client_bg.wasm",
        get_service(ServeFile::new("dist/wasm/client_bg.wasm")).handle_error(handle_error),
    );

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler() -> Html<String> {
    Html(
        TEMPLATE
            .replace("%app.root%", &render_to_string(|cx| app::App(cx)))
            .replace(
                "%app.script%",
                &format!(
                    "<script type=\"module\">{}</script>",
                    include_str!("../dist/wasm/init.min.js")
                ),
            ),
    )
}

async fn handle_error(_err: std::io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}
