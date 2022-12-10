use data::*;
use spis_model::{Media, MediaListParams, MediaType};
use sycamore::futures::spawn_local_scoped;
use sycamore::prelude::*;
use sycamore::suspense::Suspense;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

mod api;
mod data;
mod scroll;

const API_MEDIA_PER_REQ: usize = 100;

fn render_thumbnail<G: Html>(cx: Scope<'_>, media: MediaDataEntry) -> View<G> {
    let media_preview_signal = use_context::<RcSignal<Option<MediaDataEntry>>>(cx);

    let media_data = create_signal(cx, media.clone());
    let preview_display = |_| {
        media_preview_signal.set(Some(media_data.get().as_ref().clone()));
    };

    view!( cx,
        li {
          img(src=media.media.thumbnail, class="media-thumbnail", loading="lazy", on:click=preview_display) {}
        }
    )
}

#[component]
async fn MediaPreview<G: Html>(cx: Scope<'_>) -> View<G> {
    // Setup signals
    let media_list = use_context::<RcSignal<MediaData>>(cx);
    let media_preview = use_context::<RcSignal<Option<MediaDataEntry>>>(cx);
    let archive_color = create_signal(cx, "white");

    let preview_close = |_| {
        archive_color.set("white");
        media_preview.set(None);
    };

    let preview_previous = |_| {
        let index = media_preview.get().as_ref().as_ref().unwrap().index - 1;
        let prev = media_list.get().get(index).unwrap().clone();
        archive_color.set("white");
        media_preview.set(Some(prev));
    };

    let preview_next = |_| {
        let index = media_preview.get().as_ref().as_ref().unwrap().index + 1;
        let prev = media_list.get().get(index).unwrap().clone();
        archive_color.set("white");
        media_preview.set(Some(prev));
    };

    let archive = move |_| {
        let uuid = media_preview
            .get()
            .as_ref()
            .as_ref()
            .unwrap()
            .media
            .uuid
            .clone();

        spawn_local_scoped(cx, async move {
            let confirm_color = "red";
            if archive_color.get().as_ref().contains(confirm_color) {
                api::media_edit(
                    &uuid,
                    spis_model::MediaEditParams {
                        archive: Some(true),
                    },
                )
                .await
                .unwrap();

                let index = media_preview.get().as_ref().as_ref().unwrap().index;
                let old_media = media_list.get().as_ref().clone();
                let old_media = old_media.safe_remove(index);
                let next = old_media.get(index).map(|e| e.clone());
                media_list.set(old_media);
                archive_color.set("white");
                media_preview.set(next)
            } else {
                archive_color.set("red");
            }
        })
    };

    view! { cx,
        div {
            (if media_preview.get().is_some() {
                let media_type = media_preview.get().as_ref().as_ref().unwrap().media.media_type.clone();
                let media_index = media_preview.get().as_ref().as_ref().unwrap().index.clone();
                let media_total = media_preview.get().as_ref().as_ref().unwrap().total.clone();
                let media_prev = media_index > 0;
                let media_next = media_index + 1 != media_total;
                view! { cx,
                    div(class="media-preview") {
                        div(class="media-preview-content", on:click=preview_close) {
                            ({
                                let location = media_preview.get().as_ref().as_ref().unwrap().media.location.clone();
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
                        div(class="media-action") {
                            ({
                                if media_prev {
                                    view! {cx,
                                        div(class="media-action-button", on:click=preview_previous) {
                                            svg(xmlns="http://www.w3.org/2000/svg", height="24", width="24") {
                                                path(
                                                    fill="white",
                                                    d="M10 22 0 12 10 2l1.775 1.775L3.55 12l8.225 8.225Z"
                                                )
                                            }
                                        }
                                    }
                                } else {
                                    view! {cx,
                                        div(class="media-action-button") {
                                            svg(xmlns="http://www.w3.org/2000/svg", height="24", width="24") {}
                                        }
                                    }
                                }
                            })
                            div(class="media-action-button", on:click=archive) {
                                svg(xmlns="http://www.w3.org/2000/svg", height="24", width="24") {
                                    path(
                                        fill=archive_color,
                                        d="M7 21q-.825 0-1.412-.587Q5 19.825 5 19V6H4V4h5V3h6v1h5v2h-1v13q0 .825-.587 1.413Q17.825 21 17 21ZM17 6H7v13h10ZM9 17h2V8H9Zm4 0h2V8h-2ZM7 6v13Z"
                                    )
                                }
                            }
                            div(class="media-action-button", on:click=preview_close) {
                                svg(xmlns="http://www.w3.org/2000/svg", height="24", width="24") {
                                    path(
                                        fill="white",
                                        d="M6.4 19 5 17.6l5.6-5.6L5 6.4 6.4 5l5.6 5.6L17.6 5 19 6.4 13.4 12l5.6 5.6-1.4 1.4-5.6-5.6Z"
                                    )
                                }
                            }
                            ({
                                if media_next {
                                    view! {cx,
                                        div(class="media-action-button", on:click=preview_next) {
                                            svg(xmlns="http://www.w3.org/2000/svg", height="24", width="24") {
                                                path(
                                                    fill="white",
                                                    d="M8.025 22 6.25 20.225 14.475 12 6.25 3.775 8.025 2l10 10Z"
                                                )
                                            }
                                        }
                                    }
                                } else {
                                    view! {cx,
                                        div(class="media-action-button") {
                                            svg(xmlns="http://www.w3.org/2000/svg", height="24", width="24") {}
                                        }
                                    }
                                }
                            })
                        }
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
    let media_list = use_context::<RcSignal<MediaData>>(cx);
    view! { cx,
        ul(class="media-list") {
            Indexed(
                iterable=media_list,
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
async fn App<G: Html>(cx: Scope<'_>) -> View<G> {
    // Setup global preview context
    let media_preview_signal: RcSignal<Option<MediaDataEntry>> = create_rc_signal(None);
    provide_context(cx, media_preview_signal);

    // Setup media list signal, and fetch the first data
    let media_list: RcSignal<MediaData> = create_rc_signal(
        api::media_list(spis_model::MediaListParams {
            page_size: API_MEDIA_PER_REQ,
            archived: None,
            taken_after: None,
            taken_before: None,
        })
        .await
        .unwrap()
        .to_media_data(),
    );
    provide_context(cx, media_list.clone());

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
