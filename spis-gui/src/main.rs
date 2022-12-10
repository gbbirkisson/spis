use data::*;
use spis_model::{Media, MediaListParams, MediaType};
use sycamore::prelude::*;
use sycamore::suspense::Suspense;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

mod api;
mod data;
mod scroll;

const API_MEDIA_PER_REQ: usize = 100;

type IconColor<'a> = &'a str;

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

fn preview_set_previous(
    media_list: &RcSignal<MediaData>,
    media_preview: &RcSignal<Option<MediaDataEntry>>,
    archive_color: &RcSignal<IconColor>,
) {
    if media_preview.get().is_none() {
        return;
    }

    let index = media_preview.get().as_ref().as_ref().unwrap().index;
    if index == 0 {
        return;
    }

    archive_color.set("white");
    let prev = media_list.get().get(index - 1).map(|e| e.clone());
    media_preview.set(prev);
}

fn preview_set_next(
    media_list: &RcSignal<MediaData>,
    media_preview: &RcSignal<Option<MediaDataEntry>>,
    archive_color: &RcSignal<IconColor>,
) {
    if media_preview.get().is_none() {
        return;
    }

    let index = media_preview.get().as_ref().as_ref().unwrap().index + 1;
    let prev = media_list.get().get(index).map(|e| e.clone());
    if prev.is_none() {
        return;
    }
    archive_color.set("white");
    media_preview.set(prev);
}

fn preview_close(
    media_preview: &RcSignal<Option<MediaDataEntry>>,
    archive_color: &RcSignal<IconColor>,
) {
    archive_color.set("white");
    media_preview.set(None);
}

fn preview_archive<'a>(
    media_list: &'a RcSignal<MediaData>,
    media_preview: &'a RcSignal<Option<MediaDataEntry>>,
    archive_color: &'a RcSignal<IconColor>,
) {
    if media_preview.get().is_none() {
        return;
    }

    let uuid = media_preview
        .get()
        .as_ref()
        .as_ref()
        .unwrap()
        .media
        .uuid
        .clone();

    let media_list = media_list.clone();
    let media_preview = media_preview.clone();
    let archive_color = archive_color.clone();

    let confirm_color = "red";
    if !archive_color.get().as_ref().contains(confirm_color) {
        archive_color.set("red");
    } else {
        let index = media_preview.get().as_ref().as_ref().unwrap().index;
        let old_media = media_list.get().as_ref().clone();
        let old_media = old_media.safe_remove(index);
        let next = old_media.get(index).map(|e| e.clone());
        media_list.set(old_media);
        archive_color.set("white");
        media_preview.set(next);
        spawn_local(async move {
            api::media_edit(
                &uuid,
                spis_model::MediaEditParams {
                    archive: Some(true),
                },
            )
            .await
            .unwrap();
        });
    }
}

#[component]
async fn MediaPreview<G: Html>(cx: Scope<'_>) -> View<G> {
    // Setup signals
    let media_list = use_context::<RcSignal<MediaData>>(cx);
    let media_preview = use_context::<RcSignal<Option<MediaDataEntry>>>(cx);
    let archive_color = use_context::<RcSignal<IconColor>>(cx);

    let preview_close = |_| {
        preview_close(media_preview, archive_color);
    };

    let preview_previous = |_| {
        preview_set_previous(media_list, media_preview, archive_color);
    };

    let preview_next = |_| {
        preview_set_next(media_list, media_preview, archive_color);
    };

    let archive = move |_| {
        preview_archive(media_list, media_preview, archive_color);
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
    provide_context(cx, media_preview_signal.clone());

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

    let icon_archive_color: RcSignal<IconColor> = create_rc_signal("white");
    provide_context(cx, icon_archive_color.clone());

    let media_load_more = create_rc_signal(true);

    let scroll_closure_media_list = media_list.clone();

    // Create scrolling callback
    let scroll_closure: Closure<dyn FnMut()> = Closure::new(move || {
        let media_list = scroll_closure_media_list.clone();
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
            scroll_closure.as_ref().unchecked_ref(),
            false,
        )
        .expect("Failed to set listener");
    scroll_closure.forget();

    let keyboard_closure_media_list = media_list.clone();
    let keyboard_closure_media_preview_signal = media_preview_signal.clone();
    let keyboard_closure_icon_archive_color = icon_archive_color.clone();
    let keyboard_callback: Closure<dyn FnMut(_)> =
        Closure::new(move |e: web_sys::KeyboardEvent| {
            let archive_color = keyboard_closure_icon_archive_color.clone();
            let media_preview = keyboard_closure_media_preview_signal.clone();
            let media_list = keyboard_closure_media_list.clone();
            match e.key().as_str() {
                "ArrowRight" => preview_set_next(&media_list, &media_preview, &archive_color),
                "ArrowLeft" => preview_set_previous(&media_list, &media_preview, &archive_color),
                "Escape" => preview_close(&media_preview, &archive_color),
                "Delete" => preview_archive(&media_list, &media_preview, &archive_color),
                _ => (),
            }
        });
    window
        .add_event_listener_with_callback_and_bool(
            "keydown",
            keyboard_callback.as_ref().unchecked_ref(),
            false,
        )
        .expect("Failed to set listener");
    keyboard_callback.forget();

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
