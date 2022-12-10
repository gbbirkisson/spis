use reqwasm::http::Request;
use spis_model::Media;

pub(crate) async fn fetch_media_list(
    params: spis_model::MediaListParams,
) -> Result<Vec<Media>, reqwasm::Error> {
    let url = format!("/api?{}", serde_qs::to_string(&params).unwrap());
    let res = Request::get(&url).send().await?;
    let body = res.json::<Vec<Media>>().await?;
    Ok(body)
}
