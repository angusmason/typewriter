use leptos::*;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <div class="h-full flex flex-col">
            <div data-tauri-drag-region class="fixed top-0 h-30 w-full bg-white cursor-grab z-10" />
            <main class="bg-brown flex-grow m-0 pt-10 flex flex-col justify-start">
                <div
                    class="font-mono text-lg text-white caret-white bg-brown border-none outline-none pl-16 pt-4 pr-16 w-full h-full"
                    contenteditable="true"
                    spellcheck="false"
                ></div>
            </main>
        </div>
    }
}
