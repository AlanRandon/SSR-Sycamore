use app::{sycamore::render_to_string, AppProps, ClientMessage, ServerMessage};
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, get_service},
    Router,
};
use lazy_static::lazy_static;
use std::{
    net::SocketAddr,
    sync::atomic::{AtomicI64, Ordering},
};
use tokio::sync::{mpsc, Mutex};
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
    static ref WEBSOCKETS: Mutex<Vec<mpsc::Sender<Message>>> = Mutex::new(Vec::new());
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

    println!("listening on https://{}", addr);
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
        println!("New client connected");

        let (sender, mut reciever) = mpsc::channel(1);

        let sender_index = {
            let mut websockets = WEBSOCKETS.lock().await;
            websockets.push(sender);
            websockets.len() - 1
        };

        loop {
            tokio::select! {
                // Server wants to send a message
                message = reciever.recv() => {
                    if let Some(message) = message {
                        let _ = socket.send(message).await;
                    }
                }
                // Client sent a message
                message = socket.recv() => 'a: {
                    if let Some(message) = message {
                        let Ok(message) = message else {
                            // client disconnected
                            let _ = socket.close().await;
                            WEBSOCKETS.lock().await.remove(sender_index);
                            return;
                        };

                        // If the client wants to be gone, make them gone.
                        if let Message::Close(_) = message {
                            let _ = socket.close().await;
                            WEBSOCKETS.lock().await.remove(sender_index);
                            println!("Client connection closed");
                            return;
                        }

                        let Message::Binary(message) = message else {
                            break 'a;
                        };

                        let Ok(message) = postcard::from_bytes(&message) else {
                            break 'a;
                        };

                        match message {
                            ClientMessage::Increment => {
                                COUNT.fetch_add(1, Ordering::SeqCst);
                            }
                            ClientMessage::Decrement => {
                                COUNT.fetch_add(-1, Ordering::SeqCst);
                            }
                        }

                        let data =
                            postcard::to_stdvec(&ServerMessage::Set(COUNT.load(Ordering::SeqCst)))
                                .expect("Failed to serialize server message");

                        // Send an update message to all clients
                        for (index, sender) in WEBSOCKETS.lock().await.iter().enumerate() {
                            if index != sender_index {
                                let _ = sender.send(Message::Binary(data.clone())).await;
                            }
                        }
                    }
                }
            };
        }
    })
}

async fn handle_error(_err: std::io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}
