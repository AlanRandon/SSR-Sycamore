#![warn(clippy::pedantic, clippy::nursery)]

use serde::{Deserialize, Serialize};
use std::{future::Future, rc::Rc};
pub use sycamore;
use sycamore::{futures, prelude::*, rt::Event, web};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_sockets::{self, Message};

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

type EventCallback<'a> = Box<dyn 'a + FnMut(Event)>;

#[wasm_bindgen]
extern "C" {
    async fn sleep(delay: f32);
}

fn get_updates<'a>(
    cx: Scope<'a>,
    count: &'a Signal<i64>,
) -> (EventCallback<'a>, EventCallback<'a>) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        (Box::new(|_| {}), Box::new(|_| {}))
    }

    #[cfg(target_arch = "wasm32")]
    {
        let client = Rc::new(
            wasm_sockets::PollingClient::new("ws://127.0.0.1:8080/ws")
                .expect("Failed to connect to websocket"),
        );

        log::info!("Connected to websocket");

        let mut reciever = Rc::clone(&client);

        futures::spawn_local_scoped(cx, async move {
            loop {
                // for message in reciever.receive() {
                //     let Message::Binary(data) = message else {
                //         continue
                //     };

                //     if let Ok(message) = postcard::from_bytes::<ServerMessage>(&data) {
                //         log::info!("Recieved message from server: {:#?}", message);
                //         match message {
                //             ServerMessage::Set(value) => {
                //                 count.set(value);
                //             }
                //         }
                //     }
                // }
                sleep(100.0).await;
            }
        });

        let increment = {
            let client = Rc::clone(&client);
            move |_| {
                log::info!("Sending request to increment to server");
                client
                    .send_binary(postcard::to_stdvec(&ClientMessage::Increment).unwrap())
                    .unwrap_or_else(|_| log::error!("WebSocket failed to send message"));
            }
        };

        let decrement = {
            let client = Rc::clone(&client);
            move |_| {
                log::info!("Sending request to decrement to server");
                client
                    .send_binary(postcard::to_stdvec(&ClientMessage::Decrement).unwrap())
                    .unwrap_or_else(|_| log::error!("WebSocket failed to send message"));
            }
        };

        (Box::new(increment), Box::new(decrement))
    }
}

#[component]
#[must_use]
pub fn App<G: Html>(cx: Scope, props: AppProps) -> View<G> {
    let count = create_signal(cx, props.count);

    let (increment, decrement) = get_updates(cx, count);

    view! { cx,
        div(class="text-white/70 p-4 flex gap-4 items-center") {
            button(class="bg-slate-800 rounded shadow p-4 transition-colors hover:bg-slate-900", on:click=decrement) { "-" }
            p { "Count: " (count.get()) }
            button(class="bg-slate-800 rounded shadow p-4 transition-colors hover:bg-slate-900", on:click=increment) { "+" }
        }
    }
}
