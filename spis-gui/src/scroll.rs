use spis_model::{Media, MediaListParams};
use sycamore::reactive::{create_rc_signal, RcSignal};
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

use crate::{
    api::{self, API_MEDIA_PER_REQ},
    data::{MediaData, ToMediaData},
};

const PAGE_PX_LEFT_TO_FETCH_MORE: f64 = 500.0;

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

pub fn initialize(window: &web_sys::Window, media_list: RcSignal<MediaData>) {
    let media_load_more = create_rc_signal(true);
    let scroll_closure: Closure<dyn FnMut()> = Closure::new(move || {
        let media_list = media_list.clone();
        let media_load_more = media_load_more.clone();
        spawn_local(async move {
            if !media_load_more.get().as_ref() {
                return;
            }

            // So we don't do multiple requests at a time
            media_load_more.set(false);

            if at_end_of_page() {
                let old_media = media_list.get();
                let mut new_media: Vec<Media> =
                    Vec::with_capacity(old_media.len() + API_MEDIA_PER_REQ);

                for entry in old_media.iter() {
                    new_media.push(entry.media.clone());
                }

                let taken_before = new_media.last().map(|i| i.taken_at);
                let mut fetched_media = api::media_list(MediaListParams {
                    page_size: API_MEDIA_PER_REQ,
                    archived: None,
                    taken_after: None,
                    taken_before,
                })
                .await
                .unwrap(); // TODO

                let at_the_end = fetched_media.len() != API_MEDIA_PER_REQ;

                new_media.append(&mut fetched_media);
                let new_media = new_media.to_media_data();
                media_list.set(new_media);

                if !at_the_end {
                    media_load_more.set(true);
                }
            } else {
                media_load_more.set(true)
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
