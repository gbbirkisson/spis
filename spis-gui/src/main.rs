use spis_model::{Media, MediaSearchParams, MediaType};
use sycamore::prelude::*;
use sycamore::suspense::Suspense;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

mod api;
mod scroll;

const API_MEDIA_PER_REQ: usize = 100;

struct MediaPreviewData {
    location: String,
    media_type: MediaType,
}

fn render_thumbnail<G: Html>(cx: Scope<'_>, media: Media) -> View<G> {
    let media_preview_signal = use_context::<RcSignal<Option<MediaPreviewData>>>(cx);

    // This is a "private" signal to access media to create previews
    let media_data = create_signal(cx, media.clone());

    let preview_display = |_| {
        // Called when a thumbnail is pressed
        let media_data_location = media_data.get().as_ref().location.clone();
        let media_data_type = media_data.get().as_ref().media_type.clone();
        media_preview_signal.set(Some(MediaPreviewData {
            location: media_data_location,
            media_type: media_data_type,
        }));
    };

    view!( cx,
        li {
          img(src=media.thumbnail, class="media-thumbnail", loading="lazy", on:click=preview_display) {}
        }
    )
}

#[component]
async fn MediaPreview<G: Html>(cx: Scope<'_>) -> View<G> {
    // Get preview signal
    let media_preview = use_context::<RcSignal<Option<MediaPreviewData>>>(cx);

    // Setup preview close handler
    let preview_close = |_| {
        media_preview.set(None);
    };

    view! { cx,
        div {
            (if media_preview.get().is_some() {
                let media_type = media_preview.get().as_ref().as_ref().unwrap().media_type.clone();
                view! { cx,
                    div(class="media-preview", on:click=preview_close) {
                        ({
                            let location = media_preview.get().as_ref().as_ref().unwrap().location.clone();
                            match media_type {
                                MediaType::Image => view! {cx,
                                    img(class="img-preview", src=location) {}
                                },
                                MediaType::Video => view! {cx,
                                    video(class="img-preview", autoplay=true, controls=true) {
                                        source(type="video/mp4", src=location) {}
                                    }
                                }
                            }
                    })
                    }
                }
            } else {
                view! { cx, } // Show nothing
            })
        }
    }
}

#[component]
async fn MediaList<G: Html>(cx: Scope<'_>) -> View<G> {
    // Setup media list signal, and fetch the first data
    let media_list: RcSignal<Vec<Media>> = create_rc_signal(
        api::fetch_media_list(spis_model::MediaSearchParams {
            page_size: API_MEDIA_PER_REQ,
            taken_after: None,
        })
        .await
        .unwrap(),
    );
    provide_context(cx, media_list.clone());
    let media_ref = create_ref(cx, media_list.clone());

    let media_load_more = create_rc_signal(true);

    // Create scrolling callback
    let callback: Closure<dyn FnMut()> = Closure::new(move || {
        let media_list = media_list.clone();
        let media_load_more = media_load_more.clone();

        spawn_local(async move {
            if !media_load_more.get().as_ref() {
                return;
            }

            // So we don't do multiple requests at a time
            media_load_more.set(false);

            if scroll::at_end_of_page() {
                let old_media = media_list.get();
                let mut new_media: Vec<Media> =
                    Vec::with_capacity(old_media.len() + API_MEDIA_PER_REQ);

                for media in old_media.iter() {
                    new_media.push(media.clone());
                }

                let taken_after = new_media.last().map(|i| i.taken_at);
                let mut fetched_media = api::fetch_media_list(MediaSearchParams {
                    page_size: API_MEDIA_PER_REQ,
                    taken_after,
                })
                .await
                .unwrap(); // TODO

                let at_the_end = fetched_media.len() != API_MEDIA_PER_REQ;

                new_media.append(&mut fetched_media);
                media_list.set(new_media);

                if !at_the_end {
                    media_load_more.set(true);
                }
            } else {
                media_load_more.set(true)
            }
        });
    });

    // Setup callback
    let window = web_sys::window().expect("Failed to get window");
    window
        .add_event_listener_with_callback_and_bool(
            "scroll",
            callback.as_ref().unchecked_ref(),
            false,
        )
        .expect("Failed to set listener");
    callback.forget();

    view! { cx,
        ul(class="media-list") {
            Indexed(
                iterable=media_ref,
                view=|cx, media| render_thumbnail(cx, media),
            )
        }
    }
}

#[component]
fn MediaLoading<G: Html>(cx: Scope) -> View<G> {
    view! { cx,
        p {
            "Loading..."
        }
    }
}

#[component]
fn App<G: Html>(cx: Scope) -> View<G> {
    // Setup global preview context
    let media_preview_signal: RcSignal<Option<MediaPreviewData>> = create_rc_signal(None);
    provide_context(cx, media_preview_signal);

    view! { cx,
        Suspense(fallback=view! { cx, MediaLoading {} }) {
            MediaPreview {}
        }
        div(class="media-galley") {
            Suspense(fallback=view! { cx, MediaLoading {} }) {
                MediaList {}
            }
        }
    }
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();
    sycamore::render(|cx| view! { cx, App {} });
}
