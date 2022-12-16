#![warn(clippy::pedantic, clippy::nursery)]

use serde::{Deserialize, Serialize};
use std::{rc::Rc, sync::mpsc::channel};
pub use sycamore;
use sycamore::{futures, prelude::*, rt::Event};
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

#[cfg(target_arch = "wasm32")]
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
        let mut client = wasm_sockets::PollingClient::new("ws://127.0.0.1:8080/ws")
            .expect("Failed to connect to websocket");

        log::info!("Connected to websocket");

        let (sender, reciever) = channel();
        let sender = Rc::new(sender);

        futures::spawn_local_scoped(cx, async move {
            loop {
                for message in client.receive() {
                    let Message::Binary(data) = message else {
                        continue
                    };

                    if let Ok(message) = postcard::from_bytes::<ServerMessage>(&data) {
                        log::info!("Recieved message from server: {:#?}", message);
                        match message {
                            ServerMessage::Set(value) => {
                                count.set(value);
                            }
                        }
                    }
                }

                if let Ok(message) = reciever.try_recv() {
                    client
                        .send_binary(message)
                        .unwrap_or_else(|_| log::error!("Failed to send message"));
                }

                sleep(100.0).await;
            }
        });

        let increment = {
            let sender = Rc::clone(&sender);
            move |_| {
                log::info!("Sending request to increment to server");
                count.set(*count.get() + 1);
                sender
                    .send(postcard::to_stdvec(&ClientMessage::Increment).unwrap())
                    .unwrap();
            }
        };

        let decrement = {
            let sender = Rc::clone(&sender);
            move |_| {
                log::info!("Sending request to decrement to server");
                count.set(*count.get() - 1);
                sender
                    .send(postcard::to_stdvec(&ClientMessage::Decrement).unwrap())
                    .unwrap();
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
        div(class="text-white/70 p-4 flex gap-4 items-center grow") {
            button(class="bg-slate-800 rounded shadow p-4 transition transform hover:bg-slate-900 border-y-2 border-transparent hover:border-b-primary-500 hover:border-t-0 hover:border-b-2", on:click=decrement) { "-" }
            p { "Count: " (count.get()) }
            button(class="bg-slate-800 rounded shadow p-4 transition transform hover:bg-slate-900 border-y-2 border-transparent hover:border-b-primary-500 hover:border-t-0 hover:border-b-2", on:click=increment) { "+" }
        }
        footer(class="text-white/70 p-4 grid gap-4 bg-slate-800 w-full place-self-end") {
            p { "A Website" }
        }
    }
}
