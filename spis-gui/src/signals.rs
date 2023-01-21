use sycamore::reactive::{create_rc_signal, provide_context, use_context, RcSignal, Scope};

use crate::{
    data::loader::MediaDataState,
    data::{IconColor, MediaData, MediaDataEntry},
    filters::GuiFilter,
};

#[derive(Clone)]
pub struct AppSignals {
    pub media_preview: RcSignal<Option<MediaDataEntry>>,
    pub media_list: RcSignal<MediaData>,
    pub media_data_state: RcSignal<MediaDataState>,
    pub icon_archive_color: RcSignal<IconColor>,
    pub filter: RcSignal<Option<GuiFilter>>,
}

pub fn initialize(cx: Scope<'_>) -> RcSignal<AppSignals> {
    let media_preview: RcSignal<Option<MediaDataEntry>> = create_rc_signal(None);
    provide_context(cx, media_preview.clone());

    let media_list: RcSignal<MediaData> = create_rc_signal(Vec::with_capacity(0));
    provide_context(cx, media_list.clone());

    let media_data_state = create_rc_signal(MediaDataState::new());
    provide_context(cx, media_data_state.clone());

    let icon_archive_color: RcSignal<IconColor> = create_rc_signal("white".to_string());
    provide_context(cx, icon_archive_color.clone());

    let filter: RcSignal<Option<GuiFilter>> = create_rc_signal(None);
    provide_context(cx, filter.clone());

    let app_state = create_rc_signal(AppSignals {
        media_preview,
        media_list,
        media_data_state,
        icon_archive_color,
        filter,
    });
    provide_context(cx, app_state.clone());

    app_state
}

pub fn get_signals(cx: Scope<'_>) -> &RcSignal<AppSignals> {
    use_context::<RcSignal<AppSignals>>(cx)
}
