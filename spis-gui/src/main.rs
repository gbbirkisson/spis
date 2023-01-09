use data::*;
use spis_model::{MediaListParams, MediaType};
use sycamore::prelude::*;
use sycamore::suspense::Suspense;

use crate::constants::API_MEDIA_PER_REQ;
use crate::dataz::media_list_set_filter;

mod api;
mod constants;
mod data;
mod dataz;
mod motions;
mod preview;
mod signals;

fn render_thumbnail<G: Html>(cx: Scope<'_>, media: MediaDataEntry) -> View<G> {
    let signals = signals::get_signals(cx);

    let media_data = create_signal(cx, media.clone());
    let preview_open = |_| {
        preview::open(signals, media_data.get().as_ref().clone());
    };

    view!( cx,
        li {
            img(src=media.media.thumbnail, class="media-thumbnail", loading="lazy", on:click=preview_open) {}
            ({ if media.media.media_type == MediaType::Video {
                view!( cx,
                    div(class="media-thumbnail-vid", on:click=preview_open) {
                        svg(xmlns="http://www.w3.org/2000/svg", height="48", width="48") {
                            path(
                                fill="gainsboro",
                                d="M18.95 32.85 32.9 24l-13.95-8.9ZM24 45.05q-4.35 0-8.2-1.625-3.85-1.625-6.725-4.5Q6.2 36.05 4.575 32.2 2.95 28.35 2.95 24t1.625-8.2q1.625-3.85 4.5-6.725Q11.95 6.2 15.8 4.55q3.85-1.65 8.15-1.65 4.4 0 8.275 1.65t6.725 4.525q2.85 2.875 4.5 6.725 1.65 3.85 1.65 8.25 0 4.3-1.65 8.15-1.65 3.85-4.525 6.725-2.875 2.875-6.725 4.5-3.85 1.625-8.2 1.625Zm0-4.55q6.85 0 11.675-4.825Q40.5 30.85 40.5 24q0-6.85-4.825-11.675Q30.85 7.5 24 7.5q-6.85 0-11.675 4.825Q7.5 17.15 7.5 24q0 6.85 4.825 11.675Q17.15 40.5 24 40.5ZM24 24Z"
                            )
                        }
                    }
                )
                } else {
                    view!( cx, )
                }
            })
            ({ if media.media.favorite {
                view!( cx,
                    div(class="media-thumbnail-fav", on:click=preview_open) {
                        svg(xmlns="http://www.w3.org/2000/svg", height="24", width="24") {
                            path(
                                fill="pink",
                                d="m12 21-1.45-1.3q-2.525-2.275-4.175-3.925T3.75 12.812Q2.775 11.5 2.388 10.4 2 9.3 2 8.15 2 5.8 3.575 4.225 5.15 2.65 7.5 2.65q1.3 0 2.475.55T12 4.75q.85-1 2.025-1.55 1.175-.55 2.475-.55 2.35 0 3.925 1.575Q22 5.8 22 8.15q0 1.15-.387 2.25-.388 1.1-1.363 2.412-.975 1.313-2.625 2.963-1.65 1.65-4.175 3.925Z"
                            )
                        }
                    }
                )
                } else {
                    view!( cx, )
                }
            })
        }
    )
}

#[component]
async fn MediaPreview<G: Html>(cx: Scope<'_>) -> View<G> {
    let signals = signals::get_signals(cx);

    let preview_close = |_| {
        preview::close(signals);
    };

    let preview_previous = |_| {
        preview::set_previous(signals);
    };

    let preview_next = |_| {
        preview::set_next(signals);
    };

    let preview_archive = move |_| {
        preview::archive(signals);
    };

    let preview_favorite = move |_| {
        preview::favorite(signals);
    };

    let media_preview = use_context::<RcSignal<Option<MediaDataEntry>>>(cx);
    let archive_color = use_context::<RcSignal<IconColor>>(cx);

    view! { cx,
        div {
            (if media_preview.get().is_some() {
                let media_type = media_preview.get().as_ref().as_ref().unwrap().media.media_type.clone();
                let media_index = media_preview.get().as_ref().as_ref().unwrap().index;
                let media_total = media_preview.get().as_ref().as_ref().unwrap().total;
                let media_favorite = media_preview.get().as_ref().as_ref().unwrap().media.favorite;
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
                            div(class="media-action-button", on:click=preview_archive) {
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
                                if media_favorite {
                                    view! {cx,
                                        div(class="media-action-button", on:click=preview_favorite) {
                                            svg(xmlns="http://www.w3.org/2000/svg", height="24", width="24") {
                                                path(
                                                    fill="pink",
                                                    d="m12 21-1.45-1.3q-2.525-2.275-4.175-3.925T3.75 12.812Q2.775 11.5 2.388 10.4 2 9.3 2 8.15 2 5.8 3.575 4.225 5.15 2.65 7.5 2.65q1.3 0 2.475.55T12 4.75q.85-1 2.025-1.55 1.175-.55 2.475-.55 2.35 0 3.925 1.575Q22 5.8 22 8.15q0 1.15-.387 2.25-.388 1.1-1.363 2.412-.975 1.313-2.625 2.963-1.65 1.65-4.175 3.925Z"
                                                )
                                            }
                                        }
                                    }
                                } else {
                                    view! {cx,
                                        div(class="media-action-button", on:click=preview_favorite) {
                                            svg(xmlns="http://www.w3.org/2000/svg", height="24", width="24") {
                                                path(
                                                    fill="white",
                                                    d="m12 21-1.45-1.3q-2.525-2.275-4.175-3.925T3.75 12.812Q2.775 11.5 2.388 10.4 2 9.3 2 8.15 2 5.8 3.575 4.225 5.15 2.65 7.5 2.65q1.3 0 2.475.55T12 4.75q.85-1 2.025-1.55 1.175-.55 2.475-.55 2.35 0 3.925 1.575Q22 5.8 22 8.15q0 1.15-.387 2.25-.388 1.1-1.363 2.412-.975 1.313-2.625 2.963-1.65 1.65-4.175 3.925Zm0-2.7q2.4-2.15 3.95-3.688 1.55-1.537 2.45-2.674.9-1.138 1.25-2.026.35-.887.35-1.762 0-1.5-1-2.5t-2.5-1q-1.175 0-2.175.662-1 .663-1.375 1.688h-1.9q-.375-1.025-1.375-1.688-1-.662-2.175-.662-1.5 0-2.5 1t-1 2.5q0 .875.35 1.762.35.888 1.25 2.026.9 1.137 2.45 2.674Q9.6 16.15 12 18.3Zm0-6.825Z"
                                                )
                                            }
                                        }
                                    }
                                }
                            })
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
    let signals = signals::initialize(cx);

    media_list_set_filter(
        &signals,
        MediaListParams {
            page_size: API_MEDIA_PER_REQ,
            archived: None,
            favorite: None,
            taken_after: None,
            taken_before: None,
        },
    )
    .await;

    let window = web_sys::window().expect("Failed to get window");
    motions::scroll::initialize(&window, signals.clone());
    motions::swipe::initialize(&window, signals.clone());
    motions::keyboard::initialize(&window, signals.clone());

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
