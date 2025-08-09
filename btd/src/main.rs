use dioxus::prelude::*;
use dioxus_html::input_data::keyboard_types::Key;

use crate::todos::TodoItem;
mod todos;

const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        NewEntry {}
        TodoDisplay {}
        Status {}
    }
}

#[component]
fn SingleItem(todo: TodoItem) -> Element {
    let mut checked = use_signal(|| todo.completed);
    rsx! {
        div {
            class: "item-entry",
            label {
                r#for: "todo-checkbox", // Link the label to the input using the 'for' attribute
                class: "checkbox-label",
                "{todo.description}" // Label text
            }
            input {
                id: "todo-checkbox",
                r#type: "checkbox",
                checked,
                oninput: move |event| {
                    if event.checked() {
                        todo.mark_completed();
                    } else {
                        todo.mark_incomplete();
                    }
                },
            }
        }
    }
}

#[component]
fn TodoDisplay() -> Element {
    let mut test: Vec<String> = Vec::new();
    test.push("Do some work".to_string());
    rsx! {
        div {
            id: "current-todos",
            ul {
                for t in &test {
                    SingleItem { todo: TodoItem::new(1, t.clone()) }
                }
            }
        }
    }
}

#[component]
fn Status() -> Element {
    let mut status_message = use_signal(|| String::new());
    status_message.set(String::from("Default Status..."));
    rsx! {
        div {
            id: "status",
            p { "{status_message}" }
        }
    }
}

#[component]
fn NewEntry() -> Element {
    let mut new_todo = use_signal(|| String::new());
    rsx! {
        div {
            class: "newentry",
            input {
                r#type: "text",
                placeholder: "What needs to be done?",
                value: "{new_todo}",
                // As the user enters the todo text, update the variable
                oninput: move |event| {
                    new_todo.set(event.value().to_string());
                },
                // What do we do when the user presses enter?
                onkeydown: move |event| {
                    if event.key() == Key::Enter {
                        println!("Creating todos")
                    }
                }
            }
        }
    }
}

#[component]
fn Items() -> Element {
    rsx! {
        div {
            class: "item",
        }
    }
}
