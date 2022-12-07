use reqwasm::http::Request;
use spis_model::Media;

pub(crate) async fn fetch_media_list(
    params: spis_model::MediaSearchParams,
) -> Result<Vec<Media>, reqwasm::Error> {
    let url = match params.taken_before {
        None => format!("/api?page_size={}", params.page_size),
        Some(taken_before) => format!(
            "/api?page_size={}&taken_before={}",
            params.page_size, taken_before
        ),
    };
    let res = Request::get(&url).send().await?;
    let body = res.json::<Vec<Media>>().await?;
    Ok(body)
}
