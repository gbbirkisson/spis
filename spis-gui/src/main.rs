use reqwasm::http::Request;
use spis_model::Image;
use sycamore::prelude::*;
use sycamore::suspense::Suspense;
use wasm_bindgen_futures::spawn_local;

const PAGE_SIZE: usize = 10;

async fn fetch_images() -> Result<Vec<Image>, reqwasm::Error> {
    let res = Request::get("/api/").send().await?;
    let body = res.json::<Vec<Image>>().await?;
    Ok(body)
}

fn render_image<G: Html>(cx: Scope<'_>, image: Image) -> View<G> {
    view!( cx,
        li {
          img(src=image.thumbnail, loading="lazy") {}
        }
    )
}

#[component]
async fn Images<G: Html>(cx: Scope<'_>) -> View<G> {
    let images = use_context::<RcSignal<Vec<Image>>>(cx);
    let image_set = images.clone();

    spawn_local(async move {
        let old_images = image_set.get();
        let mut new_images: Vec<Image> = Vec::with_capacity(old_images.len() + PAGE_SIZE);

        for image in old_images.iter() {
            new_images.push(image.clone());
        }

        new_images.append(&mut fetch_images().await.unwrap());
        image_set.set(new_images);
    });

    view! { cx,
        ul(class="image-gallery") {
            Indexed(
                iterable=images,
                view=|cx, image| render_image(cx, image),
            )
        }
    }
}

#[component]
fn App<G: Html>(cx: Scope) -> View<G> {
    // Setup context
    let images: RcSignal<Vec<Image>> = create_rc_signal(vec![]);
    provide_context(cx, images);

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
