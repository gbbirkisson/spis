use sycamore::reactive::RcSignal;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

use crate::{
    constants::PAGE_PX_LEFT_TO_FETCH_MORE, data::loader::media_list_fetch_more, signals::AppSignals,
};

pub(crate) fn at_end_of_page() -> bool {
    let window = web_sys::window().expect("Failed to get window");
    let document = window.document().expect("Failed to get document");
    let body = document.body().expect("Failed to get body");

    let win_inner_height = window
        .inner_height()
        .expect("Failed to get window.innerHeight")
        .as_f64()
        .expect("Failed to convert window.innerHeight");
    let win_page_y_offset = window
        .page_y_offset()
        .expect("Failed to get window.pageYOffset");
    let body_offset_height = f64::from(body.offset_height());

    win_inner_height + win_page_y_offset >= body_offset_height - PAGE_PX_LEFT_TO_FETCH_MORE
}

pub fn initialize(window: &web_sys::Window, signals: RcSignal<AppSignals>) {
    let scroll_closure: Closure<dyn FnMut()> = Closure::new(move || {
        let signals = signals.clone();
        spawn_local(async move {
            if at_end_of_page() {
                media_list_fetch_more(&signals).await;
            }
        });
    });
    window
        .add_event_listener_with_callback_and_bool(
            "scroll",
            scroll_closure.as_ref().unchecked_ref(),
            false,
        )
        .expect("Failed to set listener");
    scroll_closure.forget();
}
