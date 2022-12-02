use app::{sycamore::render_to_string, AppProps};
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, get_service},
    Router,
};
use lazy_static::lazy_static;
use std::net::SocketAddr;
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
    let props = AppProps { count: 50 };

    Html(
        TEMPLATE
            .replace(
                "%app.props%",
                &base64::encode(postcard::to_stdvec(&props).unwrap_or_else(|_| Vec::new())),
            )
            .replace("%app.root%", &render_to_string(|cx| app::App(cx, props))),
    )
}

async fn handle_error(_err: std::io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}
