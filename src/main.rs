use app::sycamore::render_to_string;
use axum::{
    http::{self, StatusCode},
    response::{self, Html, IntoResponse, Response},
    routing::get,
    Router,
};
use macros::try_include_str;
use std::net::SocketAddr;

const TEMPLATE: &str = include_str!("../index.html");

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(handler))
        .route("/client_bg.wasm", get(wasm));

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
                    try_include_str!("dist/wasm/init.min.js")
                ),
            ),
    )
}

async fn wasm() -> Response<()> {}
