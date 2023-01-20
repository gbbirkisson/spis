use spis_model::{Media, MediaListParams};
use sycamore::reactive::RcSignal;

use crate::{data::ToMediaData, signals::AppSignals};

use super::api;

#[derive(Clone)]
pub struct MediaDataState {
    params: MediaListParams,
    at_end: bool,
    currently_fetching: bool,
    first_fetch: bool,
}

impl MediaDataState {
    pub(crate) fn new() -> Self {
        Self {
            params: MediaListParams::default(),
            at_end: false,
            currently_fetching: false,
            first_fetch: true,
        }
    }
}

pub(crate) async fn media_list_set_filter(signals: &RcSignal<AppSignals>, params: MediaListParams) {
    let state = signals.get().media_data_state.clone();
    state.set(MediaDataState {
        params,
        at_end: false,
        currently_fetching: false,
        first_fetch: true,
    });
    media_list_fetch_more(signals).await;
}

pub(crate) async fn media_list_fetch_more(signals: &RcSignal<AppSignals>) {
    let media = signals.get().media_list.clone();
    let state = signals.get().media_data_state.clone();

    if state.get().at_end {
        return;
    }

    let mut new_state = state.get().as_ref().clone();

    if new_state.currently_fetching {
        return;
    }

    new_state.currently_fetching = true;
    state.set(new_state.clone());

    let old_media = media.get();

    if old_media.len() > 0 && !new_state.first_fetch {
        new_state.params.taken_before = Some(
            old_media
                .last()
                .expect("Failed getting last element")
                .media
                .taken_at,
        );
    }

    let mut new_media: Vec<Media> =
        Vec::with_capacity(old_media.len() + new_state.params.page_size);

    if !new_state.first_fetch {
        for entry in old_media.iter() {
            new_media.push(entry.media.clone());
        }
    }

    let mut fetched_media = api::media_list(&new_state.params)
        .await
        .expect("API calls never fail...do they"); // TODO: FIX

    if fetched_media.len() != new_state.params.page_size {
        new_state.at_end = true
    }

    new_media.append(&mut fetched_media);
    new_state.currently_fetching = false;
    new_state.first_fetch = false;

    media.set(new_media.to_media_data());
    state.set(new_state)
}
