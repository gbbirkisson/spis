use chrono::{DateTime, Utc};
use reqwasm::http::Request;
use spis_model::Image;
use sycamore::futures::spawn_local_scoped;
use sycamore::prelude::*;
use sycamore::suspense::Suspense;

const PAGE_SIZE: usize = 50;

async fn fetch_images(prev: Option<DateTime<Utc>>) -> Result<Vec<Image>, reqwasm::Error> {
    let url = match prev {
        None => format!("/api?page_size={}", PAGE_SIZE),
        Some(prev) => format!("/api?page_size={}&prev={}", PAGE_SIZE, prev),
    };
    let res = Request::get(&url).send().await?;
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
    let more_button_visible = use_context::<RcSignal<String>>(cx);

    let load_more = move |_| {
        spawn_local_scoped(cx, async move {
            let old_images = images.get();
            let mut new_images: Vec<Image> = Vec::with_capacity(old_images.len() + PAGE_SIZE);

            for image in old_images.iter() {
                new_images.push(image.clone());
            }

            let prev = new_images.last().map(|i| i.created_at);
            let mut fetched_images = fetch_images(prev).await.unwrap(); // TODO

            if fetched_images.len() != PAGE_SIZE {
                more_button_visible.set("hide".to_string());
            }

            new_images.append(&mut fetched_images);
            images.set(new_images);
        });
    };

    view! { cx,
        ul(class="image-gallery") {
            Indexed(
                iterable=images,
                view=|cx, image| render_image(cx, image),
            )
        }
        button(class=more_button_visible, on:click=load_more) { "More" }
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
