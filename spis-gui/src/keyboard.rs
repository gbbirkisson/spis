use sycamore::reactive::RcSignal;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;

use crate::{
    data::{IconColor, MediaData, MediaDataEntry},
    preview,
};

pub fn initialize(
    window: &web_sys::Window,
    media_list: RcSignal<MediaData>,
    media_preview: RcSignal<Option<MediaDataEntry>>,
    icon_archive_color: RcSignal<IconColor>,
) {
    let keyboard_callback: Closure<dyn FnMut(_)> =
        Closure::new(move |e: web_sys::KeyboardEvent| match e.key().as_str() {
            "ArrowRight" => preview::set_next(&media_list, &media_preview, &icon_archive_color),
            "ArrowLeft" => preview::set_previous(&media_list, &media_preview, &icon_archive_color),
            "Escape" => preview::close(&media_preview, &icon_archive_color),
            "Enter" => preview::favorite(&media_list, &media_preview, &icon_archive_color),
            "Delete" => preview::archive(&media_list, &media_preview, &icon_archive_color),
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
