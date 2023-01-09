use chrono::{DateTime, Utc};
use sycamore::reactive::{create_rc_signal, RcSignal};
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;

use crate::{preview, signals::AppSignals};

pub const SWIPE_LENGTH_PIXELS_TRESHOLD: i32 = 150;
pub const SWIPE_TIME_MS_MAX: i64 = 300;

#[derive(Debug)]
enum Swipe {
    Left,
    Right,
}

#[derive(Clone)]
struct SwipeState {
    start_x: i32,
    start_time: DateTime<Utc>,
}

impl SwipeState {
    fn new(x: i32) -> Self {
        Self {
            start_x: x,
            start_time: Utc::now(),
        }
    }

    fn get(&self, x: i32) -> Option<Swipe> {
        let duration = Utc::now() - self.start_time;
        if duration > chrono::Duration::milliseconds(SWIPE_TIME_MS_MAX) {
            return None;
        }

        let diff_x = self.start_x - x;
        if diff_x.abs() < SWIPE_LENGTH_PIXELS_TRESHOLD {
            None
        } else if diff_x > 0 {
            Some(Swipe::Left)
        } else {
            Some(Swipe::Right)
        }
    }
}

pub fn initialize(window: &web_sys::Window, signals: RcSignal<AppSignals>) {
    let swipe_state_1: RcSignal<Option<SwipeState>> = create_rc_signal(None);
    let swipe_state_2 = swipe_state_1.clone();

    let touchstart_callback: Closure<dyn FnMut(_)> = Closure::new(move |e: web_sys::TouchEvent| {
        let swipe_state = swipe_state_1.clone();
        let changed = e.changed_touches().item(0);

        if changed.is_none() {
            swipe_state.set(None);
            return;
        }
        let changed = changed.unwrap();

        swipe_state.set(Some(SwipeState::new(changed.page_x())));
    });

    let touchend_callback: Closure<dyn FnMut(_)> = Closure::new(move |e: web_sys::TouchEvent| {
        let swipe_state = swipe_state_2.clone();

        if swipe_state.get().is_none() {
            return;
        }

        let changed = e.changed_touches().item(0);
        if changed.is_none() {
            swipe_state.set(None);
            return;
        }
        let changed = changed.unwrap();

        let swipe = swipe_state.get().as_ref().as_ref().cloned().unwrap();
        let swipe = swipe.get(changed.page_x());

        if let Some(swipe) = swipe {
            match swipe {
                Swipe::Right => preview::set_previous(&signals),
                Swipe::Left => preview::set_next(&signals),
            }
        }

        swipe_state.set(None);
    });

    window
        .add_event_listener_with_callback_and_bool(
            "touchstart",
            touchstart_callback.as_ref().unchecked_ref(),
            false,
        )
        .expect("Failed to set listener");
    touchstart_callback.forget();

    window
        .add_event_listener_with_callback_and_bool(
            "touchend",
            touchend_callback.as_ref().unchecked_ref(),
            false,
        )
        .expect("Failed to set listener");
    touchend_callback.forget();
}
