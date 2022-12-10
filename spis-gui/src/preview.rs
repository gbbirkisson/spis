use sycamore::reactive::RcSignal;
use wasm_bindgen_futures::spawn_local;

use crate::{
    api,
    data::{IconColor, MediaData, MediaDataEntry, SafeRemove},
};

pub fn set_previous(
    media_list: &RcSignal<MediaData>,
    media_preview: &RcSignal<Option<MediaDataEntry>>,
    archive_color: &RcSignal<IconColor>,
) {
    if media_preview.get().is_none() {
        return;
    }

    let index = media_preview.get().as_ref().as_ref().unwrap().index;
    if index == 0 {
        return;
    }

    archive_color.set("white".to_string());
    let prev = media_list.get().get(index - 1).cloned();
    media_preview.set(prev);
}

pub fn set_next(
    media_list: &RcSignal<MediaData>,
    media_preview: &RcSignal<Option<MediaDataEntry>>,
    archive_color: &RcSignal<IconColor>,
) {
    if media_preview.get().is_none() {
        return;
    }

    let index = media_preview.get().as_ref().as_ref().unwrap().index + 1;
    let prev = media_list.get().get(index).cloned();
    if prev.is_none() {
        return;
    }
    archive_color.set("white".to_string());
    media_preview.set(prev);
}

pub fn close(
    media_preview: &RcSignal<Option<MediaDataEntry>>,
    archive_color: &RcSignal<IconColor>,
) {
    archive_color.set("white".to_string());
    media_preview.set(None);
}

pub fn archive<'a>(
    media_list: &'a RcSignal<MediaData>,
    media_preview: &'a RcSignal<Option<MediaDataEntry>>,
    archive_color: &'a RcSignal<IconColor>,
) {
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

    let media_list = media_list.clone();
    let media_preview = media_preview.clone();
    let archive_color = archive_color.clone();

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
                spis_model::MediaEditParams {
                    archive: Some(true),
                },
            )
            .await
            .unwrap();
        });
    }
}
