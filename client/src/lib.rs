use app::{sycamore::hydrate, App};
use postcard::from_bytes;
use wasm_bindgen::prelude::*;
use web_sys::window;

#[wasm_bindgen(start)]
pub fn main() {
    console_log::init_with_level(log::Level::Debug).unwrap();
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    let props_string = window()
        .unwrap()
        .document()
        .unwrap()
        .body()
        .unwrap()
        .get_attribute("data-app-props")
        .unwrap();

    let props = from_bytes(&base64::decode(props_string).unwrap()).unwrap();

    log::info!("Started app with props = {:#?}", props);

    hydrate(|cx| App(cx, props));
}
