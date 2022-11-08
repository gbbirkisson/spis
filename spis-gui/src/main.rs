use gloo_net::http::Request;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
struct ImageListProps {
    images: Vec<spis_model::Image>,
}

#[function_component(ImageList)]
fn videos_list(ImageListProps { images }: &ImageListProps) -> Html {
    images
        .iter()
        .map(|image| {
            html! {
                <img src={image.thumbnail.to_string()}/>
            }
        })
        .collect()
}

#[function_component(App)]
fn app() -> Html {
    let images = use_state(Vec::new);

    {
        #[allow(clippy::redundant_clone)]
        let images = images.clone();
        use_effect_with_deps(
            move |_| {
                #[allow(clippy::redundant_clone)]
                let images = images.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let fetched_images: Vec<spis_model::Image> = Request::get("/api/")
                        .send()
                        .await
                        .unwrap()
                        .json()
                        .await
                        .unwrap();
                    images.set(fetched_images);
                });
                || ()
            },
            (),
        );
    }

    html! {
        <div>
            <ImageList images={(*images).clone()} />
        </div>
    }
}

fn main() {
    yew::start_app::<App>();
}
