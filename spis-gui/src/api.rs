use reqwasm::http::Request;
use spis_model::Media;

pub(crate) async fn fetch_media_list(
    params: spis_model::MediaSearchParams,
) -> Result<Vec<Media>, reqwasm::Error> {
    let url = match params.taken_after {
        None => format!("/api?page_size={}", params.page_size),
        Some(taken_after) => format!(
            "/api?page_size={}&taken_after={}",
            params.page_size, taken_after
        ),
    };
    let res = Request::get(&url).send().await?;
    let body = res.json::<Vec<Media>>().await?;
    Ok(body)
}
