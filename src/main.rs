#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::must_use_candidate)]
use std::path::PathBuf;
use std::str::FromStr;

use codee::string::FromToStringCodec;
use serde::de::DeserializeOwned;
use serde::Serialize;

use console_error_panic_hook::set_once;
use leptos::ev::{keydown, keyup};
use leptos::{
    component, create_action, create_effect, create_rw_signal, event_target_value, provide_context, spawn_local, use_context, window_event_listener, Action, AttributeValue, Callback, Children, CollectView, IntoView, RwSignal, Show, Signal, SignalGet, SignalGetUntracked, SignalSet, WriteSignal
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

    async fn quit() {
        invoke_without_args("quit").await;
    }
}

#[derive(Clone)]
struct Context {
    text: RwSignal<String>,
    save_path: (Signal<String>, WriteSignal<String>),
    save: Action<bool, ()>,
    original_text: RwSignal<String>,
}

#[component]
pub fn App() -> impl IntoView {
    let text = create_rw_signal(String::new());
    let original_text = create_rw_signal(String::new());
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
            original_text.set(text.get_untracked());
        }
    });
    provide_context(Context {
        text,
        save_path: (read_save_path, write_save_path),
        save,
        original_text,
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
    #[cfg(not(debug_assertions))]
    {
        use leptos::ev::contextmenu;
        window_event_listener(contextmenu, move |event| {
            event.prevent_default();
        });
    }

    view! {
        <Vertical class="h-full text-white bg-brown caret-white [&_*]:[font-synthesis:none]">
            <div data-tauri-drag-region class="absolute top-0 z-10 w-full h-12" />
            <textarea
                class="p-12 pb-72 pt-20 px-24 text-sm bg-transparent outline-none resize-none grow selection:bg-darkbrown"
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
    let Context { save_path: (read_save_path, _), save, text , original_text} = use_context().unwrap();
    let command_pressed = RwSignal::new(false);
    create_effect(move |_| {
        spawn_local({
            async move {
                let Some(data) = Inter::load_file(Some(read_save_path.get_untracked())).await
                else {
                    return;
                };
                text.set(data.clone());
                original_text.set(data); // Set the loaded text as original
            }
        });
    });
    window_event_listener(keydown, move |event| {
        if event.meta_key() {
            command_pressed.set(true);
        }
    });
    window_event_listener(keyup, move |event| {
        command_pressed.set(false);
    });
    view! {
        <div class="cursor-default px-24 inset-x-0 bottom-0 p-4 pt-6 text-xs text-right select-none text-fade">
            <Horizontal class="justify-between">
                <div class="h-6">
                    <div class="absolute transition" class=("opacity-0", command_pressed)>
                        {move || {
                            let path = PathBuf::from_str(&read_save_path())
                                .ok()
                                .map(|p| p.to_string_lossy().to_string());
                            let formatted_path = path
                                .map_or_else(
                                    String::new,
                                    |p| {
                                        let is_dirty = text.get() != original_text.get();
                                        let asterisk = if is_dirty {
                                            "<span class='text-white'> *</span> "
                                        } else {
                                            ""
                                        };
                                        format!("{p}{asterisk}")
                                    },
                                );
                            view! {
                                // Check for unsaved changes
                                <span inner_html=formatted_path />
                            }
                        }}
                    </div>
                    <div
                        class="absolute transition"
                        class=("opacity-0", move || !command_pressed())
                    >
                        <Horizontal gap=2>
                            {[
                                (
                                    vec!["c", "S"],
                                    "Save",
                                    Callback::new(move |()| save.dispatch(false)),
                                ),
                                (
                                    vec!["c", "sh", "S"],
                                    "Save as",
                                    Callback::new(move |()| save.dispatch(true)),
                                ),
                                (
                                    vec!["c", "Q"],
                                    "Quit",
                                    Callback::new(move |()| spawn_local(Inter::quit())),
                                ),
                            ]
                                .into_iter()
                                .map(|(keys, name, action)| {
                                    view! {
                                        <Horizontal gap=2>
                                            <div>{keys.join("-")}</div>
                                            <div class="text-red">{name}</div>
                                        </Horizontal>
                                    }
                                })
                                .collect_view()}
                        </Horizontal>
                    </div>
                </div>
                <Show when=move || { !text().is_empty() } fallback=|| view! { <div /> }>
                    {move || {
                        let text = text();
                        format!(
                            "{lines}L {words}W {chars}C",
                            lines = text.lines().count(),
                            words = text.split_whitespace().count(),
                            chars = text.graphemes(true).count(),
                        )
                    }}
                </Show>
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
