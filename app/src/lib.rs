#![warn(clippy::pedantic, clippy::nursery)]

use serde::{Deserialize, Serialize};
use std::rc::Rc;
pub use sycamore;
use sycamore::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_sockets::{self, WebSocketError};

#[derive(Prop, Serialize, Deserialize, Clone, Debug, Copy)]
pub struct AppProps {
    pub count: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug, Copy)]
pub enum ClientMessage {
    Increment,
    Decrement,
}

#[derive(Serialize, Deserialize, Clone, Debug, Copy)]
pub enum ServerMessage {
    Set(i64),
}

#[component]
#[must_use]
pub fn App<G: Html>(cx: Scope, props: AppProps) -> View<G> {
    let count = create_signal(cx, props.count);

    #[cfg(target_arch = "wasm32")]
    let mut client = wasm_sockets::EventClient::new("ws://127.0.0.1:8080/ws").unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        client.set_on_connection(Some(Box::new(|client: &wasm_sockets::EventClient| {
            log::info!("WebSocket Connected");
        })));

        client.set_on_message(Some(Box::new(
            |client: &wasm_sockets::EventClient, message: wasm_sockets::Message| {
                log::info!("New Message: {:#?}", message);
            },
        )));
    }

    #[cfg(target_arch = "wasm32")]
    let mut increment_send = Rc::new(move |data| client.send_binary(data));

    let increment = move |_| {
        #[cfg(target_arch = "wasm32")]
        send(vec![1]);
        count.set(*count.get() + 1);
    };

    let decrement = move |_| {
        #[cfg(target_arch = "wasm32")]
        send(vec![1]);
        count.set(*count.get() - 1);
    };

    view! { cx,
        div(class="text-white/70 p-4 flex gap-4 items-center") {
            button(class="bg-slate-800 rounded shadow p-4 transition-colors hover:bg-slate-900", on:click=decrement) { "-" }
            p { "Count: " (count.get()) }
            button(class="bg-slate-800 rounded shadow p-4 transition-colors hover:bg-slate-900", on:click=increment) { "+" }
        }
    }
}
