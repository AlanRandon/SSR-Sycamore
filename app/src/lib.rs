#![warn(clippy::pedantic, clippy::nursery)]

use serde::{Deserialize, Serialize};
pub use sycamore;
use sycamore::prelude::*;

#[derive(Prop, Serialize, Deserialize, Clone, Debug, Copy)]
pub struct AppProps {
    pub count: i32,
}

#[component]
#[must_use]
pub fn App<G: Html>(cx: Scope, props: AppProps) -> View<G> {
    let count = create_signal(cx, props.count);
    let increment = |_| count.set(*count.get() + 1);
    let decrement = |_| count.set(*count.get() - 1);
    view! { cx,
        div(class="text-white/70 p-4 flex gap-4 items-center") {
            button(class="bg-slate-800 rounded shadow p-4 transition-colors hover:bg-slate-900", on:click=decrement) { "-" }
            p { "Count: " (count.get()) }
            button(class="bg-slate-800 rounded shadow p-4 transition-colors hover:bg-slate-900", on:click=increment) { "+" }
        }
    }
}
