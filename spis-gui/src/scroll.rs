const PAGE_PX_LEFT_TO_FETCH_MORE: f64 = 500.0;

pub(crate) fn at_end_of_page() -> bool {
    let window = web_sys::window().expect("Failed to get window");
    let document = window.document().expect("Failed to get document");
    let body = document.body().expect("Failed to get body");

    let win_inner_height = window
        .inner_height()
        .expect("Failed to get window.innerHeight")
        .as_f64()
        .expect("Failed to convert window.innerHeight");
    let win_page_y_offset = window
        .page_y_offset()
        .expect("Failed to get window.pageYOffset");
    let body_offset_height = f64::from(body.offset_height());

    win_inner_height + win_page_y_offset >= body_offset_height - PAGE_PX_LEFT_TO_FETCH_MORE
}
