use std::rc::Rc;

use chrono::Datelike;
use data::*;
use filters::ActiveFilter;
use spis_model::MediaType;
use sycamore::prelude::*;
use sycamore::suspense::Suspense;
use wasm_bindgen_futures::spawn_local;

use crate::data::loader::media_list_set_filter;
use crate::filters::FilterElement;

mod constants;
mod data;
mod filters;
mod motions;
mod preview;
mod signals;
mod svg;

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
                        (svg_PLAY!(cx, "gainsboro"))
                    }
                )
                } else {
                    view!( cx, )
                }
            })
            ({ if media.media.favorite {
                view!( cx,
                    div(class="media-thumbnail-fav", on:click=preview_open) {
                        (svg_FAV_WITH_FILL!(cx, "pink"))
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
                                            (svg_LEFT!(cx, "white"))
                                        }
                                    }
                                } else {
                                    view! {cx,
                                        div(class="media-action-button") {
                                            (svg_EMPTY!(cx))
                                        }
                                    }
                                }
                            })
                            div(class="media-action-button", on:click=preview_archive) {
                                (svg_TRASHCAN!(cx, archive_color))
                            }
                            div(class="media-action-button", on:click=preview_close) {
                                (svg_X!(cx, "white"))
                            }
                            ({
                                if media_favorite {
                                    view! {cx,
                                        div(class="media-action-button", on:click=preview_favorite) {
                                            (svg_FAV_WITH_FILL!(cx, "pink"))
                                        }
                                    }
                                } else {
                                    view! {cx,
                                        div(class="media-action-button", on:click=preview_favorite) {
                                            (svg_FAV_NO_FILL!(cx, "white"))
                                        }
                                    }
                                }
                            })
                            ({
                                if media_next {
                                    view! {cx,
                                        div(class="media-action-button", on:click=preview_next) {
                                            (svg_RIGHT!(cx, "white"))
                                        }
                                    }
                                } else {
                                    view! {cx,
                                        div(class="media-action-button") {
                                            (svg_EMPTY!(cx))
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

fn build_filter_list(active_filter: Rc<ActiveFilter>) -> Vec<FilterElement> {
    let mut filters = vec![];
    filters.push(FilterElement::Favorite);

    if let Some(year) = active_filter.year() {
        // TODO, bara sýna mánuði sem hafa liðið
        filters.push(FilterElement::Year(year));
        for i in (1..=12).rev() {
            filters.push(FilterElement::Month(year, i));
        }
    } else {
        let this_year = chrono::Utc::now().year() as u16;
        for i in (this_year - 12..=this_year).rev() {
            filters.push(FilterElement::Year(i));
        }
    }

    filters
}

fn render_filter<G: Html>(cx: Scope<'_>, filter_element: FilterElement) -> View<G> {
    let signals = signals::get_signals(cx);
    let filter_element_signal = create_signal(cx, filter_element.clone());

    let filter_element_class = if signals.get().active_filter.get().is_active(&filter_element) {
        "bar-filter-link-selected"
    } else {
        "bar-filter-link"
    };

    let toggle_filter = |_| {
        let filter_element = filter_element_signal.get().as_ref().clone();
        let active_filter = signals.get().active_filter.get().as_ref().clone();

        signals
            .get()
            .active_filter
            .set(active_filter.toggle(&filter_element));
    };

    if filter_element == FilterElement::Favorite {
        view! { cx,
            li(class="bar-filter-item") {
                a(class=filter_element_class, href="#", on:click=toggle_filter) {
                    (svg_FAV_WITH_FILL!(cx, "white"))
                }
            }
        }
    } else {
        view! { cx,
            li(class="bar-filter-item") {
                a(class=filter_element_class, href="#", on:click=toggle_filter) {
                    (filter_element)
                }
            }
        }
    }
}

#[component]
async fn Bar<G: Html>(cx: Scope<'_>) -> View<G> {
    let signals = signals::get_signals(cx);
    let filters: &Signal<Vec<FilterElement>> = create_signal(cx, vec![]);
    let no_filters_enabled = create_signal(cx, false);

    create_effect(cx, move || {
        let active_filter = signals.get().active_filter.get();
        no_filters_enabled.set(active_filter.nothing_set());
        filters.set(build_filter_list(active_filter));
    });

    let clear_all_filters = |_| {
        signals.get().active_filter.set(ActiveFilter::default());
    };

    view! { cx,
        div(class="bar-inner") {
            ul(class="bar-filter-list-main") {
                (if !*no_filters_enabled.get().as_ref() {
                    view! {cx,
                        Indexed(
                            iterable=filters,
                            view=|cx, filter| render_filter(cx, filter),
                        )
                        li(class="bar-filter-item") {
                            a(href="#", on:click=clear_all_filters) {
                                (svg_X!(cx, "white"))
                            }
                        }
                    }
                } else {
                    view! {cx,
                        Indexed(
                            iterable=filters,
                            view=|cx, filter| render_filter(cx, filter),
                        )
                    }
                })
            }
        }
    }
}

#[component]
async fn App<G: Html>(cx: Scope<'_>) -> View<G> {
    let signals = signals::initialize(cx);

    let window = web_sys::window().expect("Failed to get window");
    motions::scroll::initialize(&window, signals.clone());
    motions::swipe::initialize(&window, signals.clone());
    motions::keyboard::initialize(&window, signals.clone());

    // Setup automatic fetch from api when active filter is updated
    let active_filter = signals.get().active_filter.clone();
    create_effect(cx, move || {
        let signals = signals.clone();
        let active_filter = active_filter.get();
        spawn_local(async move {
            media_list_set_filter(&signals, active_filter.as_ref().into()).await;
        });
    });

    view! { cx,
        div(class="main") {
            Suspense(fallback=view! { cx, MediaLoading {} }) {
                MediaPreview {}
            }
            div(class="bar") {
                Bar {}
            }
            div(class="media-galley") {
                Suspense(fallback=view! { cx, MediaLoading {} }) {
                    MediaList {}
                }
            }
        }
    }
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();
    sycamore::render(|cx| view! { cx, App {} });
}
