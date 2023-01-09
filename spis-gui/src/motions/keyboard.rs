use sycamore::reactive::RcSignal;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;

use crate::{preview, signals::AppSignals};

pub fn initialize(window: &web_sys::Window, signals: RcSignal<AppSignals>) {
    let keyboard_callback: Closure<dyn FnMut(_)> =
        Closure::new(move |e: web_sys::KeyboardEvent| match e.key().as_str() {
            "ArrowRight" => preview::set_next(&signals),
            "ArrowLeft" => preview::set_previous(&signals),
            "Escape" => preview::close(&signals),
            "Enter" => preview::favorite(&signals),
            "Delete" => preview::archive(&signals),
            _ => {}
        });
    window
        .add_event_listener_with_callback_and_bool(
            "keyup",
            keyboard_callback.as_ref().unchecked_ref(),
            false,
        )
        .expect("Failed to set listener");
    keyboard_callback.forget();
}
