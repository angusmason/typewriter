#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::must_use_candidate)]
use std::collections::HashMap;
use std::iter::once;

use console_error_panic_hook::set_once;
use leptos::ev::keydown;
use leptos::{
    component, create_rw_signal, event_target_value, spawn_local, window_event_listener,
    AttributeValue, Children, CollectView, IntoView, SignalGetUntracked, SignalSet,
};
use leptos::{mount_to_body, view};
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

fn save(data: String) {
    spawn_local(async move {
        invoke(
            "save_file",
            to_value(&once(("data", data)).collect::<HashMap<_, _>>()).unwrap(),
        )
        .await;
    });
}

#[component]
pub fn App() -> impl IntoView {
    let text = create_rw_signal(String::new());
    window_event_listener(keydown, move |event| {
        if !event.meta_key() || event.key() != "s" {
            return;
        }
        event.prevent_default();
        let text = text.get_untracked();
        save(text);
    });
    view! {
        <Vertical class="h-full text-white bg-brown caret-white [&_*]:[font-synthesis:none]">
            <div data-tauri-drag-region class="w-full h-8" />
            <textarea
                class="p-8 px-24 text-base bg-transparent outline-none resize-none size-full selection:bg-darkbrown"
                prop:value=text
                on:input=move |event| {
                    text.set(event_target_value(&event));
                }
            />
            <div class="fixed inset-x-0 bottom-0 p-4 text-right text-fade select-none">
                <Horizontal class="justify-between">
                    <div>
                        c-S <span class="text-red">Save</span> c-Q <span class="inline text-red">Quit</span>
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
