#![warn(clippy::pedantic, clippy::nursery)]
use console_error_panic_hook::set_once;
use leptos::{component, create_rw_signal, event_target_value, IntoView, SignalSet};
use leptos::{logging::log, mount_to_body, view};
use unicode_segmentation::UnicodeSegmentation;

macro_rules! dbg {
    () => {
        log!("[{}:{}:{}]", file!(), line!(), column!())
    };
    ($val:expr $(,)?) => {
        match $val {
            tmp => {
                log!("[{}:{}:{}] {} = {:#?}",
                    file!(), line!(), column!(), stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($(dbg!($val)),+,)
    };
}

fn main() {
    set_once();
    mount_to_body(|| {
        view! { <App /> }
    });
}

#[component]
#[must_use]
pub fn App() -> impl IntoView {
    let text = create_rw_signal(String::new());
    view! {
        <div class="flex flex-col h-full text-white bg-brown caret-white [&_*]:[font-synthesis:none]">
            <div data-tauri-drag-region class="w-full h-8 cursor-grab" />
            <textarea
                class="p-4 px-16 text-lg bg-transparent outline-none size-full selection:bg-darkbrown overscroll-none"
                prop:value=text
                on:input=move |event| {
                    text.set(event_target_value(&event));
                }
            />
            <div class="fixed bottom-0 right-0 p-4 opacity-50 pointer-events-none">
                {move || { format!("{chars} characters", chars = text().graphemes(true).count()) }}
            </div>
        </div>
    }
}
