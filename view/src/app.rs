use leptos::*;

#[component]
pub fn App(cx: Scope, counter: i64) -> impl IntoView {
    return view! {cx,
        <div> {format!("Hello, from leptos and look at this! {}", counter)} </div>
    };
}

