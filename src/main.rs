#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::must_use_candidate)]
use console_error_panic_hook::set_once;
use leptos::{
    component, create_rw_signal, event_target_value, AttributeValue, Children, CollectView,
    IntoView, SignalSet,
};
use leptos::{mount_to_body, view};
use unicode_segmentation::UnicodeSegmentation;

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

#[component]
pub fn App() -> impl IntoView {
    let text = create_rw_signal(String::new());
    
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
            <div class="fixed inset-x-0 bottom-0 p-4 text-right opacity-50">
                <Horizontal class="justify-between">
                    <div class="grid grid-cols-[auto_auto] gap-1 gap-x-2">
                        {[(vec!["cmd", "s"], "save"), (vec!["cmd", "q"], "quit")]
                            .into_iter()
                            .map(|(keys, action)| {
                                view! {
                                    <div class="px-1 text-sm border border-white rounded">{keys.join(" ")}</div>
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
