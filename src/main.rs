#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::must_use_candidate)]

use std::path::PathBuf;

use codee::string::FromToStringCodec;
use serde::de::DeserializeOwned;
use serde::Serialize;

use console_error_panic_hook::set_once;
use leptos::ev::{keydown, keyup};
use leptos::{
    component, create_action, create_effect, create_rw_signal, event_target_value, provide_context,
    spawn_local, use_context, window_event_listener, Action, AttributeValue, Callback, Children,
    CollectView, IntoView, RwSignal, Show, Signal, SignalGetUntracked, SignalSet, WriteSignal,
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

    async fn save_file(data: String, path: Option<PathBuf>) -> Option<String> {
        #[derive(Serialize)]
        struct SaveFileArgs {
            data: String,
            path: Option<PathBuf>,
        }
        Self::call("save_file", &SaveFileArgs { data, path }).await
    }

    async fn load_file(path: Option<PathBuf>) -> (Option<String>, Option<PathBuf>) {
        #[derive(Serialize)]
        struct LoadFileArgs {
            path: Option<PathBuf>,
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
                Some(read_save_path.get_untracked()).filter(|path| !save_as && !path.is_empty()).map(PathBuf::from),
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
                class="px-24 pt-20 text-sm bg-transparent outline-none resize-none pb-72 grow selection:bg-darkbrown"
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
#[allow(clippy::too_many_lines)]
fn StatusBar() -> impl IntoView {
    macro_rules! shortcut {
        (c-sh-$char:expr; $name:literal => $action:block) => {
            Shortcut {
                shift: true,
                char: $char,
                name: $name,
                action: Callback::new(move |()| $action),
            }
        };
        (c-$char:expr; $name:literal => $action:block) => {
            Shortcut {
                shift: false,
                char: $char,
                name: $name,
                action: Callback::new(move |()| $action),
            }
        };
    }
    #[derive(Clone, Copy)]
    struct Shortcut {
        shift: bool,
        char: char,
        name: &'static str,
        action: Callback<()>,
    }
    let Context {
        save_path: (read_save_path, write_save_path),
        save,
        text,
        original_text,
    } = use_context().unwrap();
    let command_pressed = RwSignal::new(false);
    create_effect(move |_| {
        spawn_local({
            async move {
                let (Some(data), _) = Inter::load_file(Some(read_save_path.get_untracked().into())).await
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
    window_event_listener(keyup, move |_| {
        command_pressed.set(false);
    });
    let shortcuts = [
        shortcut!(
            c-'s';
            "Save" => {
                save.dispatch(false);
            }
        ),
        shortcut!(
            c-sh-'s';
            "Save as" => {
                save.dispatch(true);
            }
        ),
        shortcut!(
            c-'q';
            "Quit" => {
                spawn_local(Inter::quit());
            }
        ),
        shortcut!(
            c-'n';
            "New" => {
                text.set(String::new());
                write_save_path(String::new());
            }
        ),
        shortcut!(
            c-'o';
            "Open" => {
                spawn_local(async move {
                    let (Some(data), Some(path)) = Inter::load_file(None).await else {
                        return;
                    };
                    text.set(data);
                    write_save_path(path.to_str().unwrap().to_string());
                    command_pressed.set(false);
                });
            }
        ),
    ];
    window_event_listener(keydown, move |event| {
        for Shortcut {
            shift,
            char,
            action,
            ..
        } in shortcuts
        {
            if !command_pressed() {
                continue;
            }
            if shift && !event.shift_key() {
                continue;
            }
            if event.key() != char.to_string() {
                continue;
            }
            action(());
        }
    });
    view! {
        <div class="inset-x-0 bottom-0 p-4 px-24 pt-6 text-xs text-right cursor-default select-none text-fade">
            <Horizontal class="justify-between">
                <div class="h-6">
                    <div class="absolute transition" class=("opacity-0", command_pressed)>
                        <Horizontal gap=1>
                            {read_save_path}
                            <Show when=move || {
                                text() != original_text() && !read_save_path().is_empty()
                            }>
                                <div class="text-white">"*"</div>
                            </Show>
                        </Horizontal>
                    </div>
                    <div
                        class="absolute transition"
                        class=("opacity-0", move || !command_pressed())
                    >
                        <Horizontal gap=2>
                            {shortcuts
                                .into_iter()
                                .map(|Shortcut { shift, char, name, action }| {
                                    let char = char.to_ascii_uppercase();
                                    view! {
                                        <button on:click=move |_| action(())>
                                            <Horizontal gap=2 class="transition hover:brightness-150">
                                                <div>
                                                    {format!("c-{}{char}", if shift { "shift-" } else { "" })}
                                                </div>
                                                <div class="text-red">{name}</div>
                                            </Horizontal>
                                        </button>
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
