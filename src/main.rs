#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::must_use_candidate)]
use std::path::PathBuf;
use std::str::FromStr;

use codee::string::FromToStringCodec;
use serde::de::DeserializeOwned;
use serde::Serialize;

use console_error_panic_hook::set_once;
use leptos::ev::keydown;
use leptos::{
    component, create_action, create_effect, create_rw_signal, event_target_value, provide_context,
    spawn_local, use_context, window_event_listener, AttributeValue, Callback, Children,
    CollectView, IntoView, RwSignal, SignalGetUntracked, SignalSet,
};
use leptos::{mount_to_body, view};
use leptos_use::storage::use_local_storage;
use serde_wasm_bindgen::{from_value, to_value};
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

struct Inter;

#[allow(clippy::future_not_send)]
impl Inter {
    async fn call<R: DeserializeOwned>(cmd: &str, args: &impl Serialize) -> R {
        from_value(invoke(cmd, to_value(args).unwrap()).await).unwrap()
    }

    async fn save_file(data: String, path: Option<String>) -> Option<String> {
        #[derive(Serialize)]
        struct SaveFileArgs {
            data: String,
            path: Option<String>,
        }
        Self::call("save_file", &SaveFileArgs { data, path }).await
    }

    async fn load_file(path: Option<String>) -> Option<String> {
        #[derive(Serialize)]
        struct LoadFileArgs {
            path: Option<String>,
        }
        Self::call("load_file", &LoadFileArgs { path }).await
    }
}

#[component]
pub fn App() -> impl IntoView {
    let text = create_rw_signal(String::new());
    provide_context(text);
    view! {
        <Vertical class="h-full text-white bg-brown caret-white [&_*]:[font-synthesis:none]">
            <div data-tauri-drag-region class="w-full h-8" />
            <textarea
                class="p-8 px-24 text-base bg-transparent outline-none resize-none size-full selection:bg-darkbrown"
                prop:value=text
                autocorrect="off"
                on:input=move |event| {
                    text.set(event_target_value(&event));
                }
            />
            <StatusBar />
        </Vertical>
    }
}

#[component]
fn StatusBar() -> impl IntoView {
    let text: RwSignal<String> = use_context().unwrap();
    let (read_save_path, write_save_path, _) =
        use_local_storage::<String, FromToStringCodec>("save_path");
    let save = create_action(move |save_as| {
        let save_as: bool = *save_as;
        async move {
            let Some(path) = Inter::save_file(
                text.get_untracked(),
                Some(read_save_path.get_untracked()).filter(|path| !save_as && !path.is_empty()),
            )
            .await
            else {
                return;
            };
            write_save_path(path);
        }
    });
    window_event_listener(keydown, move |event| {
        if event.meta_key() && event.key() == "s" {
            event.prevent_default();
            save.dispatch(event.shift_key());
        }
    });
    create_effect(move |_| {
        spawn_local({
            async move {
                let Some(data) = Inter::load_file(Some(read_save_path.get_untracked())).await
                else {
                    return;
                };
                text.set(data);
            }
        });
    });

    window_event_listener(leptos::ev::contextmenu, move |event| {
        event.prevent_default();
    });

    view! {
        <div class="fixed inset-x-0 bottom-0 p-4 text-base text-right select-none text-fade">
            <Horizontal class="justify-between">
                {move || {
                    PathBuf::from_str(&read_save_path())
                        .ok()
                        .map(|path| { path.to_string_lossy().to_string() })
                }}
                <Horizontal gap=2>
                    {[
                        (vec!["c", "S"], "Save", Callback::new(move |()| save.dispatch(false))),
                        (
                            vec!["c", "shift", "S"],
                            "Save as",
                            Callback::new(move |()| save.dispatch(true)),
                        ),
                        (vec!["c", "Q"], "Quit", Callback::new(move |()| {})),
                    ]
                        .into_iter()
                        .map(|(keys, name, action)| {
                            view! {
                                <button on:click=move |_| action(())>
                                    <Horizontal gap=2 class="transition hover:brightness-150">
                                        <div>{keys.join("-")}</div>
                                        <div class="text-red">{name}</div>
                                    </Horizontal>
                                </button>
                            }
                        })
                        .collect_view()}
                </Horizontal>
                <div
                    class="relative *:transition group transition"
                    class=("opacity-0", move || text().is_empty())
                >
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
                    <div class="truncate opacity-0 group-hover:opacity-100">
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
