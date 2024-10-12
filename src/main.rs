#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::must_use_candidate)]

use std::array::from_fn;
use std::borrow::Cow;
use std::path::PathBuf;

use codee::string::FromToStringCodec;
use leptos::html::Div;
use serde::de::DeserializeOwned;
use serde::Serialize;

use console_error_panic_hook::set_once;
use leptos::ev::{keydown, keyup};
use leptos::{
    component, create_action, create_effect, create_memo, create_node_ref, create_rw_signal,
    event_target, event_target_value, provide_context, spawn_local, use_context,
    window_event_listener, Action, AttributeValue, Callback, Children, CollectView, For,
    HtmlElement, IntoView, RwSignal, Show, Signal, SignalGetUntracked, SignalSet, SignalUpdate,
    ViewFn, WriteSignal,
};
use leptos::{mount_to_body, view};
use leptos_use::storage::use_local_storage;
use serde_wasm_bindgen::{from_value, to_value};
use unicode_segmentation::UnicodeSegmentation;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;
use web_sys::HtmlTextAreaElement;

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

    /// Saves a file containing some data to a path, prompting the user if the path is [`None`].
    ///
    /// Returns the path to the file or [`None`] if the
    /// user cancelled the save by closing the dialog.
    ///
    /// Panics if the data couldn't be written.
    async fn save_file(data: String, path: Option<PathBuf>) -> Option<String> {
        #[derive(Serialize)]
        struct SaveFileArgs {
            data: String,
            path: Option<PathBuf>,
        }
        Self::call("save_file", &SaveFileArgs { data, path }).await
    }

    /// Loads a file from a path.
    /// Returns a tuple containing the data in the file and the path to the file.
    ///
    /// The returned data is [`None`] if the file couldn't be read (it didn't exist, or did not
    /// contain valid UTF-8) - or if the supplied path was [`None`] and the user did not provide a
    /// fallback (they cancelled the load by closing the dialog).
    ///
    /// The returned path is [`None`] if the supplied path was [`None`] and the user did not
    /// provide a fallback (they cancelled the load by closing the dialog).
    async fn load_file(path: Option<PathBuf>) -> (Option<String>, Option<PathBuf>) {
        #[derive(Serialize)]
        struct LoadFileArgs {
            path: Option<PathBuf>,
        }
        Self::call("load_file", &LoadFileArgs { path }).await
    }

    /// Quits the program. Exit code is `0` (success).
    async fn quit() {
        invoke_without_args("quit").await;
    }
}

#[derive(Clone)]
struct Context {
    text: RwSignal<String>,
    save_path: (Signal<String>, WriteSignal<String>),
    save: Action<bool, ()>,
    unsaved: RwSignal<bool>,
    selection: RwSignal<Option<(usize, usize)>>,
}

#[component]
#[allow(clippy::too_many_lines)]
pub fn App() -> impl IntoView {
    let text = create_rw_signal(String::new());
    let (read_save_path, write_save_path, _) =
        use_local_storage::<String, FromToStringCodec>("save_path");
    let unsaved = create_rw_signal(false);
    let save = create_action(move |save_as| {
        let save_as: bool = *save_as;
        async move {
            let Some(path) = Inter::save_file(
                text.get_untracked(),
                Some(read_save_path.get_untracked())
                    .filter(|path| !save_as && !path.is_empty())
                    .map(PathBuf::from),
            )
            .await
            else {
                return;
            };
            write_save_path(path);
            unsaved.set(false);
        }
    });
    let selection = create_rw_signal(None);
    provide_context(Context {
        text,
        save_path: (read_save_path, write_save_path),
        save,
        unsaved,
        selection,
    });
    #[cfg(not(debug_assertions))]
    {
        use leptos::ev::contextmenu;
        window_event_listener(contextmenu, move |event| {
            event.prevent_default();
        });
    }
    let original = create_rw_signal(None);
    create_effect(move |_| {
        let read_save_path = read_save_path();
        spawn_local(async move {
            original.set(Some(Inter::load_file(Some(read_save_path.into())).await.0));
        });
    });
    let overlay = create_node_ref();
    let sync = move |event| {
        let overlay: HtmlElement<Div> = overlay().unwrap();
        let text_area = event_target::<HtmlTextAreaElement>(&event);
        overlay.set_scroll_top(text_area.scroll_top());
    };
    view! {
        <Vertical
            class="h-full text-text bg-background caret-caret [&_*]:[font-synthesis:none] [&_*]:[font-variant-ligatures:none] px-80 pb-4"
            gap=6
        >
            <div data-tauri-drag-region class="absolute top-0 z-10 w-full h-12" />
            <div class="relative size-full">
                <div
                    class="absolute top-0 left-0 pt-20 overflow-y-auto text-sm break-words whitespace-pre-wrap size-full"
                    ref=overlay
                >
                    <div class="relative size-full">
                        <div class="absolute top-0 size-full">
                            {move || {
                                const fn position(
                                    lengths: &[usize],
                                    index: usize,
                                ) -> (usize, usize) {
                                    let mut low = 0;
                                    let mut high = lengths.len();
                                    while low < high {
                                        let mid = (low + high) / 2;
                                        if index < lengths[mid] {
                                            high = mid;
                                        } else {
                                            low = mid + 1;
                                        }
                                    }
                                    (low - 1, index - lengths[low - 1])
                                }
                                let char_to_position = move |char: usize| {
                                    let mut first = 0;
                                    let mut last = 0;
                                    for (index, line) in text().lines().enumerate() {
                                        last += line.len();
                                        if last == char {
                                            return (index, last - first)
                                        } else if char.max(last) == char {
                                            last += 1;
                                            first = last;
                                        } else {
                                            return (index, char - first)
                                        }
                                    }
                                    unreachable!()
                                };
                                let (start, end) = selection()
                                    .map(|(start, end)| (
                                        char_to_position(start),
                                        char_to_position(end),
                                    ))?;
                                Some(
                                    text()
                                        .lines()
                                        .enumerate()
                                        .map(|(index, line)| {
                                            let len = line.len();
                                            view! {
                                                <div
                                                    class="h-5"
                                                    style:padding-left=move || {
                                                        format!("{}ch", if start.0 == index { start.1 } else { 0 })
                                                    }
                                                >
                                                    <Horizontal class="h-full">
                                                        <div
                                                            class="h-full rounded bg-highlight"
                                                            style:width=move || {
                                                                format!(
                                                                    "{}ch",
                                                                    if start.0 == end.0 {
                                                                        if start.0 == index { end.1 - start.1 } else { 0 }
                                                                    } else if index > start.0 && index < end.0 {
                                                                        len
                                                                    } else if index == start.0 {
                                                                        len - start.1
                                                                    } else if index == end.0 {
                                                                        end.1
                                                                    } else {
                                                                        0
                                                                    },
                                                                )
                                                            }
                                                        ></div>
                                                        <Show when=move || {
                                                            (start.0 == end.0 && start.0 == index) || index == end.0
                                                        }>
                                                            <div class="h-full bg-caret w-0.5"></div>
                                                        </Show>
                                                    </Horizontal>
                                                </div>
                                            }
                                        })
                                        .collect_view(),
                                )
                            }}
                        </div>
                        <div class="absolute top-0 z-10 size-full">
                            {move || {
                                let text = text();
                                text.lines()
                                    .map(|line| {
                                        view! { <div class="h-5">{line.to_string()}</div> }
                                    })
                                    .collect_view()
                            }}
                        </div>
                    </div>
                </div>
                <textarea
                    class="absolute top-0 left-0 z-20 pt-20 overflow-y-auto text-sm text-transparent whitespace-pre-wrap bg-transparent outline-none resize-none size-full overscroll-none selection:bg-transparent"
                    prop:value=text
                    autocorrect="off"
                    on:input=move |event| {
                        text.set(event_target_value(&event));
                        spawn_local(async move {
                            unsaved
                                .set(
                                    original.get_untracked().flatten() != Some(text.get_untracked()),
                                );
                        });
                        sync(event);
                    }
                    on:select=move |event| {
                        let text_area: HtmlTextAreaElement = event_target(&event);
                        selection
                            .set(
                                Some((
                                    text_area.selection_start().unwrap().unwrap() as usize,
                                    text_area.selection_end().unwrap().unwrap() as usize,
                                )),
                            );
                    }
                    on:mousedown=move |_| {
                        selection.set(None);
                    }
                    on:keydown=move |event| {
                        if event.key() == "Tab" {
                            event.prevent_default();
                            let text_area = event_target::<HtmlTextAreaElement>(&event);
                            let selection = selection
                                .get_untracked()
                                .unwrap_or_else(|| (
                                    text_area.selection_start().unwrap().unwrap() as usize,
                                    text_area.selection_end().unwrap().unwrap() as usize,
                                ));
                            text.update(|text| {
                                *text = format!(
                                    "{}\t{}",
                                    &text[0..selection.0],
                                    &text[selection.1..],
                                );
                            });
                            let position = selection.0 + 1;
                            #[allow(clippy::cast_possible_truncation)]
                            {
                                text_area.set_selection_start(Some(position as u32)).unwrap();
                                text_area.set_selection_end(Some(position as u32)).unwrap();
                            }
                        }
                    }
                    on:scroll=sync
                />
            </div>
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
        unsaved,
        selection,
    } = use_context().unwrap();
    let command_pressed = RwSignal::new(false);
    create_effect(move |_| {
        spawn_local({
            async move {
                let (Some(data), _) =
                    Inter::load_file(Some(read_save_path.get_untracked().into())).await
                else {
                    return;
                };
                text.set(data);
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

    let find_text = create_rw_signal(String::new());
    let matches = create_rw_signal(Vec::new());
    let current_match_index = create_rw_signal(0);
    let show_find_input = create_rw_signal(false);

    let shortcuts = [
        shortcut!(
            c-'f';
            "Find" => {
                show_find_input.set(true);
                find_text.set(String::new());
                matches.set(Vec::new());
                current_match_index.set(0);
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
            event.prevent_default();
            action(());
        }
    });

    let find_matches = move || {
        let mut new_matches = Vec::new();
        let mut start_index = 0;

        while let Some(index) = text.get_untracked()[start_index..].find(&find_text.get_untracked())
        {
            new_matches.push(start_index + index);
            start_index += index + find_text.get_untracked().len();
        }

        matches.set(new_matches);
        current_match_index.set(0);
    };

    let move_to_next_match = move || {
        if matches.get_untracked().is_empty() {
            return;
        }
        let next_index = (current_match_index.get_untracked() + 1) % matches.get_untracked().len();
        current_match_index.set(next_index);
    };

    window_event_listener(keydown, move |event| {
        if event.key() == "Escape" && show_find_input() {
            show_find_input.set(false);
            find_text.set(String::new());
            matches.set(Vec::new());
            current_match_index.set(0);
        }
    });

    view! {
        <div class="text-xs text-right cursor-default select-none text-fade">
            <Horizontal class="justify-between">
                <div class="h-6">
                    <Match cases=[
                        (
                            (move |()| command_pressed()).into(),
                            (move || {
                                view! {
                                    <Horizontal gap=2>
                                        {shortcuts
                                            .into_iter()
                                            .map(|Shortcut { shift, char, name, .. }| {
                                                let char = char.to_ascii_uppercase();
                                                view! {
                                                    <Horizontal gap=2>
                                                        <div>
                                                            {format!("c-{}{char}", if shift { "sh-" } else { "" })}
                                                        </div>
                                                        <div class="text-accent">{name}</div>
                                                    </Horizontal>
                                                }
                                            })
                                            .collect_view()}
                                    </Horizontal>
                                }
                            })
                                .into(),
                        ),
                        (
                            (move |()| show_find_input()).into(),
                            (move || {
                                view! {
                                    <Horizontal gap=1>
                                        <div class="text-text">"find:"</div>
                                        <input
                                            type="text"
                                            class="select-text text-text bg-background cursor-text selection:bg-highlight"
                                            prop:value=find_text
                                            on:input=move |_| {
                                                find_matches();
                                            }
                                            on:keydown=move |event| {
                                                if event.key() == "Enter" {
                                                    move_to_next_match();
                                                }
                                            }
                                        />
                                    </Horizontal>
                                }
                            })
                                .into(),
                        ),
                        (
                            (move |()| !show_find_input()).into(),
                            (move || {
                                view! {
                                    <Horizontal gap=1>
                                        {move || {
                                            let path = PathBuf::from(read_save_path());
                                            let mut components = path.components();
                                            let mut components: [_; 4] = from_fn(|_| {
                                                components.next_back()
                                            });
                                            components.reverse();
                                            components
                                                .into_iter()
                                                .flatten()
                                                .collect::<PathBuf>()
                                                .to_string_lossy()
                                                .to_string()
                                        }} <Show when=unsaved>
                                            <div class="text-text">"*"</div>
                                        </Show>
                                    </Horizontal>
                                }
                            })
                                .into(),
                        ),
                    ] />
                </div>
                <Show when=move || { !text().is_empty() } fallback=|| view! { <div /> }>
                    {move || {
                        let text = Cow::from(text());
                        let text = if let Some((start, end)) = selection() {
                            text.get(start..end).unwrap_or(&text).into()
                        } else {
                            text
                        };
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

#[component]
fn Match<const N: usize>(#[prop(into)] cases: [(Callback<(), bool>, ViewFn); N]) -> impl IntoView {
    let matched = create_memo({
        let cases = cases.clone();
        move |_| cases.iter().position(|(condition, _)| condition(()))
    });
    view! {
        <For
            each=move || cases.clone().into_iter().enumerate()
            key=move |(index, _)| *index
            children=move |(index, (_, view))| {
                view! {
                    <div
                        class="absolute transition"
                        class=(
                            ["opacity-0", "pointer-events-none"],
                            move || { Some(index) != matched() },
                        )
                    >
                        {view.run()}
                    </div>
                }
            }
        />
    }
}
