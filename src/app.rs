use leptos::*;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <div>
            <div class="absolute top-0 h-30 w-full bg-slate-500" data-tauri-drag-region />
        </div>
        <main class="container">
            <div>
                <div class="absolute top-0 h-30 w-full bg-slate-500" data-tauri-drag-region />
            </div>
            <textarea contenteditable="true" style="color: #FFFFFF; background-color:#29272B"></textarea>
        </main>
    }
}
