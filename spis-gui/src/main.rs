use reqwasm::http::Request;
use spis_model::Image;
use sycamore::prelude::*;
use sycamore::suspense::Suspense;

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
    let images = fetch_images().await.unwrap();
    let images = create_signal(cx, images);

    // let window = web_sys::window().expect("could not get window handle");
    // let document = window.document().expect("could not get document handle");

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
