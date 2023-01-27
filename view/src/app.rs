use std::collections::HashMap;

use leptos::*;

#[component]
pub fn App(cx: Scope, counter: i64) -> impl IntoView {
    let mut map = HashMap::new();
    map.insert(String::from("hello"), 7);

    return view! {cx,
        <div> {format!("Hello, from leptos and look at this! {}", counter)} </div>
    };
}

