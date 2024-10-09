#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::must_use_candidate)]
use codee::string::FromToStringCodec;
use serde::Serialize;

use console_error_panic_hook::set_once;
use leptos::ev::keydown;
use leptos::{
    component, create_action, create_rw_signal, event_target_value, window_event_listener,
    AttributeValue, Children, CollectView, IntoView, SignalGetUntracked, SignalSet,
};
use leptos::{mount_to_body, view};
use leptos_use::storage::use_local_storage;
use serde_wasm_bindgen::to_value;
use unicode_segmentation::UnicodeSegmentation;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

#[allow(unused_macros)]
macro_rules! dbg {
    () => {
        leptos::logging::log!("[{}:{}:{}]", file!(), line!(), column!())
    };
    ($val:expr $(,)?) => {
        match $val {
            tmp => {
                leptos::logging::log!("[{}:{}:{}] {} = {:#?}",
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

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], js_name = invoke)]
    async fn invoke_without_args(cmd: &str) -> JsValue;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[component]
pub fn App() -> impl IntoView {
    let text = create_rw_signal(String::new());
    let (read_save_path, write_save_path, _) =
        use_local_storage::<String, FromToStringCodec>("save_path");
    let save = create_action(move |save_as| {
        let save_as: bool = *save_as;
        async move {
            #[derive(Serialize)]
            struct SaveFileArgs {
                data: String,
                path: Option<String>,
            }
            write_save_path(
                invoke(
                    "save_file",
                    to_value(&SaveFileArgs {
                        data: text.get_untracked(),
                        path: Some(read_save_path.get_untracked())
                            .filter(|path| !save_as && !path.is_empty()),
                    })
                    .unwrap(),
                )
                .await
                .as_string()
                .unwrap(),
            );
        }
    });
    window_event_listener(keydown, move |event| {
        if event.meta_key() && event.key() == "s" {
            event.prevent_default();
            save.dispatch(event.shift_key());
        }
    });
    view! {
        <Vertical class="h-full text-white bg-brown caret-white [&_*]:[font-synthesis:none]">
            <div data-tauri-drag-region class="w-full h-8 cursor-grab" />
            <textarea
                class="p-4 px-16 text-sm bg-transparent outline-none resize-none size-full selection:bg-darkbrown"
                prop:value=text
                on:input=move |event| {
                    text.set(event_target_value(&event));
                }
            />
            <div class="fixed inset-x-0 bottom-0 p-4 text-right opacity-50 select-none">
                <Horizontal class="justify-between">
                    <div class="grid grid-cols-[auto_auto] gap-1 gap-x-2">
                        {[
                            (vec!["cmd", "s"], "save"),
                            (vec!["cmd", "shift", "s"], "save as"),
                            (vec!["cmd", "q"], "quit"),
                        ]
                            .into_iter()
                            .map(|(keys, action)| {
                                view! {
                                    <Horizontal>
                                        <div class="px-1 text-sm border border-white rounded">
                                            {keys.join(" ")}
                                        </div>
                                    </Horizontal>
                                    <div class="">{action}</div>
                                }
                            })
                            .collect_view()}
                    </div>
                    <div class="relative *:transition group">
                        <div class="absolute bottom-0 right-0 truncate group-hover:opacity-0">
                            {move || {
                                let text = text();
                                format!(
                                    "{lines}L {words}W {chars}C",
                                    lines = text.lines().count(),
                                    words = text.split_whitespace().count(),
                                    chars = text.graphemes(true).count(),
                                )
                            }}
                        </div>
                        <div class="absolute bottom-0 right-0 truncate opacity-0 group-hover:opacity-100">
                            {move || {
                                let text = text();
                                format!(
                                    "{lines} lines, {words} words, {chars} characters",
                                    lines = text.lines().count(),
                                    words = text.split_whitespace().count(),
                                    chars = text.graphemes(true).count(),
                                )
                            }}
                        </div>
                    </div>
                </Horizontal>
            </div>
        </Vertical>
    }
}

#[component]
#[allow(clippy::cast_precision_loss)]
pub fn Horizontal(
    children: Children,
    #[prop(optional, into)] gap: f64,
    #[prop(optional, into)] class: Option<AttributeValue>,
) -> impl IntoView {
    view! {
        <div class=class_to_string(class) + " flex" style=format!("gap: {}rem", gap / 4.)>
            {children()}
        </div>
    }
}

#[component]
#[allow(clippy::cast_precision_loss)]
pub fn Vertical(
    children: Children,
    #[prop(optional, into)] gap: f64,
    #[prop(optional, into)] class: Option<AttributeValue>,
) -> impl IntoView {
    view! {
        <div class=class_to_string(class) + " flex flex-col" style=format!("gap: {}rem", gap / 4.)>
            {children()}
        </div>
    }
}

pub fn class_to_string(class: Option<AttributeValue>) -> String {
    class
        .map(|class| {
            class
                .into_attribute_boxed()
                .as_nameless_value_string()
                .unwrap_or_default()
        })
        .unwrap_or_default()
        .to_string()
}
