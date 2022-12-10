use data::*;
use spis_model::MediaType;
use sycamore::prelude::*;
use sycamore::suspense::Suspense;

use crate::api::API_MEDIA_PER_REQ;

mod api;
mod data;
mod keyboard;
mod preview;
mod scroll;

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
    let archive_color = use_context::<RcSignal<IconColor>>(cx);

    let preview_close = |_| {
        preview::close(media_preview, archive_color);
    };

    let preview_previous = |_| {
        preview::set_previous(media_list, media_preview, archive_color);
    };

    let preview_next = |_| {
        preview::set_next(media_list, media_preview, archive_color);
    };

    let archive = move |_| {
        preview::archive(media_list, media_preview, archive_color);
    };

    view! { cx,
        div {
            (if media_preview.get().is_some() {
                let media_type = media_preview.get().as_ref().as_ref().unwrap().media.media_type.clone();
                let media_index = media_preview.get().as_ref().as_ref().unwrap().index;
                let media_total = media_preview.get().as_ref().as_ref().unwrap().total;
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

    // Setup icon archive, color
    let icon_archive_color: RcSignal<IconColor> = create_rc_signal("white".to_string());
    provide_context(cx, icon_archive_color.clone());

    // Initialize window listeners
    let window = web_sys::window().expect("Failed to get window");
    scroll::initialize(&window, media_list.clone());
    keyboard::initialize(
        &window,
        media_list,
        media_preview_signal,
        icon_archive_color,
    );

    // Return view
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
