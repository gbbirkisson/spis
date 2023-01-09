use log::info;
use sycamore::reactive::RcSignal;
use wasm_bindgen_futures::spawn_local;

use crate::{
    api,
    data::{MediaDataEntry, SafeRemove},
    signals::AppSignals,
};

pub fn set_previous(signals: &RcSignal<AppSignals>) {
    let media_list = signals.get().media_list.clone();
    let media_preview = signals.get().media_preview.clone();
    let icon_archive_color = signals.get().icon_archive_color.clone();

    if media_preview.get().is_none() {
        return;
    }

    let index = signals
        .get()
        .media_preview
        .get()
        .as_ref()
        .as_ref()
        .unwrap()
        .index;
    if index == 0 {
        return;
    }

    icon_archive_color.set("white".to_string());
    let prev = media_list.get().get(index - 1).cloned();
    media_preview.set(prev);
}

pub fn set_next(signals: &RcSignal<AppSignals>) {
    let media_list = signals.get().media_list.clone();
    let media_preview = signals.get().media_preview.clone();
    let icon_archive_color = signals.get().icon_archive_color.clone();

    if media_preview.get().is_none() {
        return;
    }

    let index = signals
        .get()
        .media_preview
        .get()
        .as_ref()
        .as_ref()
        .unwrap()
        .index
        + 1;

    let prev = media_list.get().get(index).cloned();
    if prev.is_none() {
        return;
    }

    icon_archive_color.set("white".to_string());
    open(signals, prev.unwrap());
}

pub fn open(signals: &RcSignal<AppSignals>, media: MediaDataEntry) {
    if media.total - media.index < 3 {
        info!("asdasd");
    }
    signals.get().media_preview.set(Some(media));
}

pub fn close(signals: &RcSignal<AppSignals>) {
    signals.get().icon_archive_color.set("white".to_string());
    signals.get().media_preview.set(None);
}

pub fn archive(signals: &RcSignal<AppSignals>) {
    let media_list = signals.get().media_list.clone();
    let media_preview = signals.get().media_preview.clone();
    let archive_color = signals.get().icon_archive_color.clone();

    if media_preview.get().is_none() {
        return;
    }

    let uuid = media_preview
        .get()
        .as_ref()
        .as_ref()
        .unwrap()
        .media
        .uuid
        .clone();

    let confirm_color = "red";
    if !archive_color.get().as_ref().contains(confirm_color) {
        archive_color.set("red".to_string());
    } else {
        let index = media_preview.get().as_ref().as_ref().unwrap().index;
        let old_media = media_list.get().as_ref().clone();
        let old_media = old_media.safe_remove(index);
        let next = old_media.get(index).cloned();
        media_list.set(old_media);
        archive_color.set("white".to_string());
        media_preview.set(next);
        spawn_local(async move {
            api::media_edit(
                &uuid,
                &spis_model::MediaEditParams {
                    archive: Some(true),
                    favorite: None,
                },
            )
            .await
            .unwrap();
        });
    }
}

pub fn favorite(signals: &RcSignal<AppSignals>) {
    let media_list = signals.get().media_list.clone();
    let media_preview = signals.get().media_preview.clone();
    let icon_archive_color = signals.get().icon_archive_color.clone();

    if media_preview.get().is_none() {
        return;
    }

    let uuid = media_preview
        .get()
        .as_ref()
        .as_ref()
        .unwrap()
        .media
        .uuid
        .clone();

    icon_archive_color.set("white".to_string());

    let mut new_val = signals
        .get()
        .media_preview
        .get()
        .as_ref()
        .as_ref()
        .unwrap()
        .clone();
    new_val.media.favorite = !new_val.media.favorite;

    let mut old_media = media_list.get().as_ref().clone();

    media_preview.set(Some(new_val.clone()));
    old_media.get_mut(new_val.index).unwrap().media.favorite = new_val.media.favorite;
    media_list.set(old_media);

    spawn_local(async move {
        api::media_edit(
            &uuid,
            &spis_model::MediaEditParams {
                archive: None,
                favorite: Some(new_val.media.favorite),
            },
        )
        .await
        .unwrap();
    });
}
