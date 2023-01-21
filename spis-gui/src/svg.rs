#[macro_export]
macro_rules! svg_X {
    ($cx:expr, $color:expr) => {
        view! { $cx,
            svg(xmlns="http://www.w3.org/2000/svg", height="24", width="24") {
                path(
                    fill=$color,
                    d="M6.4 19 5 17.6l5.6-5.6L5 6.4 6.4 5l5.6 5.6L17.6 5 19 6.4 13.4 12l5.6 5.6-1.4 1.4-5.6-5.6Z"
                )
            }
        }
    };
}

#[macro_export]
macro_rules! svg_FAV_WITH_FILL {
    ($cx:expr, $color:expr) => {
        view! { $cx,
            svg(xmlns="http://www.w3.org/2000/svg", height="24", width="24") {
                path(
                    fill=$color,
                    d="m12 21-1.45-1.3q-2.525-2.275-4.175-3.925T3.75 12.812Q2.775 11.5 2.388 10.4 2 9.3 2 8.15 2 5.8 3.575 4.225 5.15 2.65 7.5 2.65q1.3 0 2.475.55T12 4.75q.85-1 2.025-1.55 1.175-.55 2.475-.55 2.35 0 3.925 1.575Q22 5.8 22 8.15q0 1.15-.387 2.25-.388 1.1-1.363 2.412-.975 1.313-2.625 2.963-1.65 1.65-4.175 3.925Z"
                )
            }
        }
    };
}

#[macro_export]
macro_rules! svg_FAV_NO_FILL {
    ($cx:expr, $color:expr) => {
        view! { $cx,
            svg(xmlns="http://www.w3.org/2000/svg", height="24", width="24") {
                path(
                    fill=$color,
                    d="m12 21-1.45-1.3q-2.525-2.275-4.175-3.925T3.75 12.812Q2.775 11.5 2.388 10.4 2 9.3 2 8.15 2 5.8 3.575 4.225 5.15 2.65 7.5 2.65q1.3 0 2.475.55T12 4.75q.85-1 2.025-1.55 1.175-.55 2.475-.55 2.35 0 3.925 1.575Q22 5.8 22 8.15q0 1.15-.387 2.25-.388 1.1-1.363 2.412-.975 1.313-2.625 2.963-1.65 1.65-4.175 3.925Zm0-2.7q2.4-2.15 3.95-3.688 1.55-1.537 2.45-2.674.9-1.138 1.25-2.026.35-.887.35-1.762 0-1.5-1-2.5t-2.5-1q-1.175 0-2.175.662-1 .663-1.375 1.688h-1.9q-.375-1.025-1.375-1.688-1-.662-2.175-.662-1.5 0-2.5 1t-1 2.5q0 .875.35 1.762.35.888 1.25 2.026.9 1.137 2.45 2.674Q9.6 16.15 12 18.3Zm0-6.825Z"
                )
            }
        }
    };
}

#[macro_export]
macro_rules! svg_LEFT {
    ($cx:expr, $color:expr) => {
        view! { $cx,
            svg(xmlns="http://www.w3.org/2000/svg", height="24", width="24") {
                path(
                    fill=$color,
                    d="M10 22 0 12 10 2l1.775 1.775L3.55 12l8.225 8.225Z"
                )
            }
        }
    };
}

#[macro_export]
macro_rules! svg_RIGHT {
    ($cx:expr, $color:expr) => {
        view! { $cx,
            svg(xmlns="http://www.w3.org/2000/svg", height="24", width="24") {
                path(
                    fill=$color,
                    d="M8.025 22 6.25 20.225 14.475 12 6.25 3.775 8.025 2l10 10Z"
                )
            }
        }
    };
}

#[macro_export]
macro_rules! svg_EMPTY {
    ($cx:expr) => {
        view! { $cx,
            svg(xmlns="http://www.w3.org/2000/svg", height="24", width="24") {}
        }
    };
}

#[macro_export]
macro_rules! svg_TRASHCAN {
    ($cx:expr, $color:expr) => {
        view! { $cx,
            svg(xmlns="http://www.w3.org/2000/svg", height="24", width="24") {
                path(
                    fill=$color,
                    d="M7 21q-.825 0-1.412-.587Q5 19.825 5 19V6H4V4h5V3h6v1h5v2h-1v13q0 .825-.587 1.413Q17.825 21 17 21ZM17 6H7v13h10ZM9 17h2V8H9Zm4 0h2V8h-2ZM7 6v13Z"
                )
            }
        }
    };
}

#[macro_export]
macro_rules! svg_PLAY {
    ($cx:expr, $color:expr) => {
        view! { $cx,
            svg(xmlns="http://www.w3.org/2000/svg", height="48", width="48") {
                path(
                    fill=$color,
                    d="M18.95 32.85 32.9 24l-13.95-8.9ZM24 45.05q-4.35 0-8.2-1.625-3.85-1.625-6.725-4.5Q6.2 36.05 4.575 32.2 2.95 28.35 2.95 24t1.625-8.2q1.625-3.85 4.5-6.725Q11.95 6.2 15.8 4.55q3.85-1.65 8.15-1.65 4.4 0 8.275 1.65t6.725 4.525q2.85 2.875 4.5 6.725 1.65 3.85 1.65 8.25 0 4.3-1.65 8.15-1.65 3.85-4.525 6.725-2.875 2.875-6.725 4.5-3.85 1.625-8.2 1.625Zm0-4.55q6.85 0 11.675-4.825Q40.5 30.85 40.5 24q0-6.85-4.825-11.675Q30.85 7.5 24 7.5q-6.85 0-11.675 4.825Q7.5 17.15 7.5 24q0 6.85 4.825 11.675Q17.15 40.5 24 40.5ZM24 24Z"
                )
            }
        }
    };
}
