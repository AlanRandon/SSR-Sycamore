use app::{sycamore::hydrate, App};
// use wasm_bindgen::prelude::*;

pub fn main() {
    console_log::init_with_level(log::Level::Debug).unwrap();
    log::info!("hello world");
    hydrate(|cx| App(cx))
}
