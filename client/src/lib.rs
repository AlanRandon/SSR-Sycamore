use app::{sycamore::hydrate, App};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    console_log::init_with_level(log::Level::Debug).unwrap();

    log::info!("Rust is running");

    hydrate(|cx| App(cx))
}
