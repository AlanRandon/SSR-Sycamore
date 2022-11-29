pub use sycamore;
use sycamore::prelude::*;

#[component]
pub fn App<G: Html>(cx: Scope) -> View<G> {
    view! { cx,
        p { "hello world" }
    }
}
