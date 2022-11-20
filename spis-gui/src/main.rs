use reqwasm::http::Request;
use spis_model::{Media, MediaSearchParams};
use sycamore::prelude::*;
use sycamore::suspense::Suspense;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

const API_MEDIA_PER_REQ: usize = 100;
const PAGE_PX_LEFT_TO_FETCH_MORE: f64 = 500.0;

struct MediaViewData {
    location: String,
}

async fn fetch_media(params: spis_model::MediaSearchParams) -> Result<Vec<Media>, reqwasm::Error> {
    let url = match params.taken_after {
        None => format!("/api/?page_size={}", params.page_size),
        Some(taken_after) => format!(
            "/api/?page_size={}&taken_after={}",
            params.page_size, taken_after
        ),
    };
    let res = Request::get(&url).send().await?;
    let body = res.json::<Vec<Media>>().await?;
    Ok(body)
}

fn render_thumbnail<G: Html>(cx: Scope<'_>, media: Media) -> View<G> {
    let media_view = use_context::<RcSignal<Option<MediaViewData>>>(cx);
    let media_view_data = create_signal(cx, media.clone());

    let preview_display = |_| {
        let location = media_view_data.get().as_ref().location.clone();
        media_view.set(Some(MediaViewData { location }));
    };

    view!( cx,
        li {
          img(src=media.thumbnail, class="thumbnail", loading="lazy", on:click=preview_display) {}
        }
    )
}

#[component]
async fn MediaView<G: Html>(cx: Scope<'_>) -> View<G> {
    let media_view = use_context::<RcSignal<Option<MediaViewData>>>(cx);

    let preview_close = |_| {
        media_view.set(None);
    };

    view! { cx,
        div {
            (if media_view.get().is_some() {
                let location = media_view.get().as_ref().as_ref().unwrap().location.clone();
                view! { cx,
                    div(class="media-view") {
                        img(class="preview", src=location, on:click=preview_close) {}
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
    // Setup signals
    let media: RcSignal<Vec<Media>> = create_rc_signal(
        fetch_media(spis_model::MediaSearchParams {
            page_size: API_MEDIA_PER_REQ,
            taken_after: None,
        })
        .await
        .unwrap(),
    );

    let media_loading = create_rc_signal(false);
    let media_reached_end = create_rc_signal(false);
    let media_ref = create_ref(cx, media.clone());

    provide_context(cx, media.clone());

    // Create scrolling callback
    let callback: Closure<dyn FnMut()> = Closure::new(move || {
        let media = media.clone();
        let media_loading = media_loading.clone();
        let media_reached_end = media_reached_end.clone();

        spawn_local(async move {
            if media_reached_end.get().as_ref() == &true {
                return;
            }

            if media_loading.get().as_ref() == &true {
                return;
            }
            media_loading.set(true);

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

            let near_end_of_page = win_inner_height + win_page_y_offset
                >= body_offset_height - PAGE_PX_LEFT_TO_FETCH_MORE;

            if near_end_of_page {
                let old_media = media.get();
                let mut new_media: Vec<Media> =
                    Vec::with_capacity(old_media.len() + API_MEDIA_PER_REQ);

                for media in old_media.iter() {
                    new_media.push(media.clone());
                }

                let taken_after = new_media.last().map(|i| i.taken_at);
                let mut fetched_media = fetch_media(MediaSearchParams {
                    page_size: API_MEDIA_PER_REQ,
                    taken_after,
                })
                .await
                .unwrap(); // TODO

                if fetched_media.len() != API_MEDIA_PER_REQ {
                    media_reached_end.set(true);
                }

                new_media.append(&mut fetched_media);
                media.set(new_media);
            }

            media_loading.set(false);
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

    // Return view
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
    let media_view: RcSignal<Option<MediaViewData>> = create_rc_signal(None);
    provide_context(cx, media_view);

    view! { cx,
            Suspense(fallback=view! { cx, MediaLoading {} }) {
                MediaView {}
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
