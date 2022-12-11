use reqwasm::http::Request;
use spis_model::Media;

pub const API_MEDIA_PER_REQ: usize = 100;

pub(crate) async fn media_list(
    params: spis_model::MediaListParams,
) -> Result<Vec<Media>, reqwasm::Error> {
    let url = format!("/api?{}", serde_qs::to_string(&params).unwrap());
    let res = Request::get(&url).send().await?;
    let body = res.json::<Vec<Media>>().await?;
    Ok(body)
}

pub(crate) async fn media_edit(
    uuid: &str,
    params: spis_model::MediaEditParams,
) -> Result<bool, reqwasm::Error> {
    let url = format!("/api/{}?{}", uuid, serde_qs::to_string(&params).unwrap());
    let res = Request::post(&url).send().await?;
    let body = res.json::<bool>().await?;
    Ok(body)
}
