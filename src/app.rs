use leptos::{component, view, IntoView};

#[component]
pub fn App() -> impl IntoView {
    view! {
        <div class="flex flex-col h-full">
            <div data-tauri-drag-region class="fixed top-0 z-10 w-full bg-white h-30 cursor-grab" />
            <main class="flex flex-col justify-start flex-grow pt-10 m-0 bg-brown">
                <div
                    class="w-full h-full pt-4 pl-16 pr-16 font-mono text-lg text-white border-none outline-none caret-white bg-brown"
                    contenteditable="true"
                    spellcheck="false"
                ></div>
            </main>
        </div>
    }
}
