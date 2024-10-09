#![warn(clippy::pedantic, clippy::nursery)]
use console_error_panic_hook::set_once;
use leptos::{component, IntoView};
use leptos::{mount_to_body, view};

fn main() {
    set_once();
    mount_to_body(|| {
        view! { <App /> }
    });
}

#[component]
#[must_use]
pub fn App() -> impl IntoView {
    view! {
        <div class="flex flex-col h-full bg-brown">
            <div data-tauri-drag-region class="fixed top-0 z-10 w-full h-8 bg-white cursor-grab" />
            <main class="flex flex-col justify-start pt-10 grow">
                <div
                    class="w-full h-full px-16 pt-4 font-mono text-lg text-white outline-none caret-white"
                    contenteditable="true"
                    spellcheck="false"
                ></div>
            </main>
        </div>
    }
}
