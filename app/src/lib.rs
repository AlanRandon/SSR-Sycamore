use serde::{Deserialize, Serialize};
pub use sycamore;
use sycamore::prelude::*;

#[derive(Prop, Serialize, Deserialize, Clone, Debug)]
pub struct AppProps {
    pub count: i32,
}

#[component]
pub fn App<G: Html>(cx: Scope, props: AppProps) -> View<G> {
    let count = create_signal(cx, props.count);
    let increment = |_| count.set(*count.get() + 1);
    let decrement = |_| count.set(*count.get() - 1);
    view! { cx,
        div {
            button(on:click=decrement) { "-" }
            p { "Count " (count.get()) }
            button(on:click=increment) { "+" }
        }
    }
}
