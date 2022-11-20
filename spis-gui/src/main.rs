use reqwasm::http::Request;
use spis_model::{Image, ImageSeachParams};
use sycamore::prelude::*;
use sycamore::suspense::Suspense;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

const API_IMAGE_PER_REQ: usize = 30;
const PAGE_PX_LEFT_TO_FETCH_MORE: f64 = 200.0;

async fn fetch_images(params: spis_model::ImageSeachParams) -> Result<Vec<Image>, reqwasm::Error> {
    let url = match params.taken_after {
        None => format!("/api/?page_size={}", params.page_size),
        Some(taken_after) => format!(
            "/api/?page_size={}&taken_after={}",
            params.page_size, taken_after
        ),
    };
    let res = Request::get(&url).send().await?;
    let body = res.json::<Vec<Image>>().await?;
    Ok(body)
}

fn render_image<G: Html>(cx: Scope<'_>, image: Image) -> View<G> {
    view!( cx,
        li {
          img(src=image.thumbnail, class="thumbnail", loading="lazy") {}
        }
    )
}

#[component]
async fn Images<G: Html>(cx: Scope<'_>) -> View<G> {
    // Setup signals
    let images: RcSignal<Vec<Image>> = create_rc_signal(
        fetch_images(spis_model::ImageSeachParams {
            page_size: API_IMAGE_PER_REQ,
            taken_after: None,
        })
        .await
        .unwrap(),
    );
    let images_loading = create_rc_signal(false);
    let images_reached_end = create_rc_signal(false);
    let images_ref = create_ref(cx, images.clone());

    // Create scrolling callback
    let callback: Closure<dyn FnMut()> = Closure::new(move || {
        let images = images.clone();
        let images_loading = images_loading.clone();
        let images_reached_end = images_reached_end.clone();

        spawn_local(async move {
            if images_reached_end.get().as_ref() == &true {
                return;
            }

            if images_loading.get().as_ref() == &true {
                return;
            }
            images_loading.set(true);

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
                let old_images = images.get();
                let mut new_images: Vec<Image> =
                    Vec::with_capacity(old_images.len() + API_IMAGE_PER_REQ);

                for image in old_images.iter() {
                    new_images.push(image.clone());
                }

                let taken_after = new_images.last().map(|i| i.taken_at);
                let mut fetched_images = fetch_images(ImageSeachParams {
                    page_size: API_IMAGE_PER_REQ,
                    taken_after,
                })
                .await
                .unwrap(); // TODO

                if fetched_images.len() != API_IMAGE_PER_REQ {
                    images_reached_end.set(true);
                }

                new_images.append(&mut fetched_images);
                images.set(new_images);
            }

            images_loading.set(false);
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
        ul(class="image-gallery") {
            Indexed(
                iterable=images_ref,
                view=|cx, image| render_image(cx, image),
            )
        }
    }
}

#[component]
fn App<G: Html>(cx: Scope) -> View<G> {
    let images: RcSignal<Vec<Image>> = create_rc_signal(vec![]);
    let more_button_visible: RcSignal<String> = create_rc_signal("".to_string());

    provide_context(cx, images);
    provide_context(cx, more_button_visible);

    view! { cx,
        div(class="container") {
            Suspense(fallback=view! { cx, "Loading..." }) {
                Images {}
            }
        }
    }
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    sycamore::render(|cx| view! { cx, App {} });
}
