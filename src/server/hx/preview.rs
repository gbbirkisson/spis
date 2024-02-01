use super::Media;
use actix_web::{delete, get, put};
use actix_web::{web, Responder};
use askama_actix::{Template, TemplateToResponse};

#[derive(Template)]
#[template(path = "preview/preview.html")]
struct HxRoot<'a> {
    media: Option<&'a Media>,
}

#[get("/{idx}")]
async fn root(path: web::Path<String>) -> impl Responder {
    let _idx = path.into_inner();

    let media = Media {
        uuid: "123123".into(),
        url: "http://stufur:1337/assets/media/tota_myndir/2018/20180723_183916.jpg".into(),
        thumbnail: "http://stufur:1337/assets/thumbnails/1601707f-b75e-3640-91e4-0c4331ec7f6e.webp"
            .into(),
        favorite: true,
        video: false,
        taken_at: chrono::offset::Utc::now(),
    };

    HxRoot {
        media: Some(&media),
    }
    .to_response()
}

#[put("/{idx}/favorite")]
async fn favorite(path: web::Path<String>) -> impl Responder {
    let _idx = path.into_inner();

    let media = Media {
        uuid: "123123".into(),
        url: "http://stufur:1337/assets/media/tota_myndir/2018/20180723_183916.jpg".into(),
        thumbnail: "http://stufur:1337/assets/thumbnails/1601707f-b75e-3640-91e4-0c4331ec7f6e.webp"
            .into(),
        favorite: false,
        video: false,
        taken_at: chrono::offset::Utc::now(),
    };

    HxRoot {
        media: Some(&media),
    }
    .to_response()
}

#[delete("/{idx}")]
async fn archive(path: web::Path<String>) -> impl Responder {
    let _idx = path.into_inner();

    HxRoot { media: None }.to_response()
}
