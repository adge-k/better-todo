use dioxus::prelude::*;
use ollama_rs::{
    Ollama, generation::{
        completion::request::GenerationRequest,
        parameters::FormatType
    }
};
use tokio_stream::StreamExt;
use serde::{Deserialize, Serialize};
use dioxus_html::input_data::keyboard_types::Key;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

const JSON_FORMAT: &str = r#"{
    "steps": [
        {
            "content": "<step>",
        }
    ]
}"#;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        TodoApp {}
    }
}

// Define the components for the API and Task Handler to work

// NOTE: PartialEq == Trait for "equality".  This means you can do "Task == Task" or "Task += Task"
// Data structures from initial_api_testing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Task {
    pub content: String,
}

// impl Task {
//     fn next(task: Task) -> Self {
//         Task { content: "Test".to_string() }
//     }
// }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Todo {
    pub steps: Vec<Task>,
}

/// Define a new Todo.  This needs to be expanded to handle parenting.
/// That way each "todo" is a single entry, but the parent level defines
/// if it's a sub-task of a main todo or not.
/// Right now, it only takes the JSON response from the LLM and adds it all
/// at once.  "from_string()" should become a loop.
impl Todo {
    // New Todo
    pub fn new() -> Self {
        Todo { steps: vec![] }
    }

    // Add a Task
    pub fn add_task(&mut self, task: Task) {
        self.steps.push(task);
    }

    pub fn from_string(s: String) -> Self {
        let mut steps = vec![];
        let cleaned_data = clean_json_string(&s);
        match serde_json::from_str::<Todo>(&cleaned_data) {
            Ok(t) => {
                for task in t.steps {
                    steps.push(Task { content: task.content });
                }
            },
            Err(e) => eprintln!("Error parsing JSON: {}", e),
        }
        Todo { steps }
    }
}

/// Useful function for cleaning JSON.  LLM response tends to include
/// a lot of \n and whitespace.  This attempts to make the final result
/// JSON serialisable.
fn clean_json_string(input: &str) -> String {
    input
        .trim()
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("")
}

/// Hard coded control prompt for now.  This should be made to be a variable later.
fn control_prompt() -> String {
    format!(
        "You are an assistant who breaks up todo items into achievable tasks.
        The resulting items must be no more than 5.
        Each item must a short and concise todo entry.
        The result must be json serialisable.
        The result must strictly match this format, including step number indexes: {}", JSON_FORMAT
    )
}

/// Enum for tracking different states of Ollama operations
/// - This was a Claude Code addition.  Makes sense in order to
/// show state in the UI.
#[derive(Debug, Clone, PartialEq)]
enum OllamaState {
    Idle,
    Generating,
    Completed(Todo),
    Error(String),
}

impl std::fmt::Display for OllamaState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OllamaState::Idle => write!(f, "Idle..."),
            OllamaState::Generating => write!(f, "Generating..."),
            OllamaState::Completed(_) => write!(f, "Completed!"),
            OllamaState::Error(e) => write!(f, "Error: {}", e),
        }
    }
}


#[component]
fn TodoApp() -> Element {
    // State management for the todo input
    let mut user_input = use_signal(|| String::new());

    // State management for Ollama operations
    let mut ollama_state = use_signal(|| OllamaState::Idle);

    // State for storing generated todos
    let mut generated_todos = use_signal(|| Vec::<Todo>::new());

    // Function to generate todos using Ollama
    let generate_todos = move |input: String| {
        spawn(async move {
            // Set state to generating which the UI elements should be disabling for
            ollama_state.set(OllamaState::Generating);

            // PSEUDOCODE: This is where the Ollama integration happens
            match generate_todo_breakdown(input).await {
                Ok(todo) => {
                    // Add to our list of generated todos
                    let mut current_todos = generated_todos.read().clone();
                    current_todos.push(todo.clone());
                    generated_todos.set(current_todos);
                    // Update state to completed
                    ollama_state.set(OllamaState::Completed(todo));
                },
                Err(e) => {
                    ollama_state.set(OllamaState::Error(format!("Failed to generate todos: {}", e)));
                }
            }
        });
    };

    rsx! {
        // Main app div.
        div {
            id: "todo-app",

            // Header
            // 
            h1 { "Better Todo" }

            // Input section
            // TODO(adge-k): This needs to become an actual todo entry.
            div {
                class: "input-section",

                // New input field.  This needs to become the todo entry itself.
                // Add to a new div
                input {
                    class: "todo-input",
                    placeholder: "New item",
                    value: "{user_input}",
                    // Make sure this remains, as it's what is filling the user_input variable.
                    // TODO(adge-k): This could probably be made more efficient by reading the final input value somehow
                    oninput: move |event| {
                        user_input.set(event.value().to_string());
                    },
                    onkeydown: move |event| {
                        if event.key() == Key::Enter {
                            println!("Enter key pressed!");
                            // Now user_input will have the latest value because oninput updates it
                            let input = user_input.read().clone();
                            println!("Detected input: {}", input);
                            if !input.trim().is_empty() {
                                println!("Input ({}) is not empty", input);
                                generate_todos(input);
                                user_input.set(String::new()); // Clear input
                            }
                        }
                    },
                    disabled: matches!(*ollama_state.read(), OllamaState::Generating),  // Disables the button if state is "Generating"
                }

                // Generated todos display
                // This should actually just be a child div of the todo it was generated from
                div {
                    class: "todos-section",
                    if !generated_todos.read().is_empty() {
                        h2 { "Generated Todo Lists" }
                        
                        for (index, todo) in generated_todos.read().iter().enumerate() {
                            TodoDisplay { 
                                todo: todo.clone(), 
                                index: index + 1 
                            }
                        }
                    } else {
                        div {
                            class: "empty-state",
                            "No todos generated yet. Enter a task above to get started!"
                        }
                    }
                }
            }
        }
        // Status indicator
        div {
            id: "status-bar",   
            p { "{ollama_state.read().to_string()}" },
        }
    }
}

#[component]
fn TodoDisplay(todo: Todo, index: usize) -> Element {
    rsx! {
        div {
            class: "todo-display",
            
            h3 { "Todo List #{index}" }
            
            ul {
                class: "task-list",
                for task in &todo.steps {
                    TaskItem { task: task.clone() }
                }
            }
        }
    }
}

#[component] 
fn TaskItem(task: Task) -> Element {
    let mut completed = use_signal(|| false);
    
    rsx! {
        li {
            class: if completed() { "task-item completed" } else { "task-item" },
            
            input {
                r#type: "checkbox",
                class: "task-checkbox",
                checked: completed(),
                onchange: move |event| completed.set(event.checked()),
            }
            
            span {
                class: "task-content",
                "{task.content}"
            }
        }
    }
}

// ASYNC FUNCTION: This is the core integration with Ollama
// This function encapsulates the Ollama API call logic from initial_api_testing
async fn generate_todo_breakdown(input: String) -> Result<Todo, Box<dyn std::error::Error + Send + Sync>> {
    // Initialize Ollama client
    let ollama = Ollama::default();
    
    // Configure the model and prompt
    let model = "gemma3n".to_string();
    let control = control_prompt();
    let prompt = format!("{}. The todo is: {}", control, input);
    
    println!("Beginning request to Ollama now...");
    // Make the streaming request
    let mut resp = ollama
        .generate_stream(GenerationRequest::new(model, prompt).format(FormatType::Json))
        .await?;
    
    println!("Response received!");
    // Collect streaming response
    let mut raw_data: Vec<String> = vec![];
    while let Some(res) = resp.next().await {
        let responses = res?;
        for r in responses {
            raw_data.push(r.response);
        }
    }
    
    // Parse the response into our Todo structure
    let todo = Todo::from_string(raw_data.join(""));
    
    println!("Final todos: {:?}", todo);

    Ok(todo)
}

/* 
INTEGRATION NOTES:

1. **State Management**: 
   - `ollama_state` tracks the current operation (Idle, Generating, Completed, Error)
   - `generated_todos` stores all successfully generated todo lists
   - `user_input` manages the input field

2. **Async Operations**:
   - `spawn()` is used to run async Ollama operations without blocking the UI
   - The UI remains responsive during LLM generation

3. **Error Handling**:
   - Comprehensive error states with user-friendly messages
   - Network failures and parsing errors are caught and displayed

4. **Required Cargo.toml additions**:
   You'll need to add these dependencies to integrated_ui_test/Cargo.toml:
   
   ```toml
   [dependencies]
   dioxus = { version = "0.6.0", features = ["fullstack"] }
   tokio = { version = "1.0", features = ["full"] }
   serde = { version = "1.0", features = ["derive"] }
   serde_json = "1.0"
   ollama-rs = { version = "0.3.2", features = ["stream"] }
   tokio-stream = "0.1.17"
   ```

5. **CSS Classes** (add to main.css):
   - .todo-app, .input-section, .status-section, .todos-section
   - .generating, .completed, .error for status styling
   - .task-item, .task-checkbox for todo items

6. **Usage Pattern**:
   - User enters a high-level todo
   - Clicks "Generate Tasks" 
   - UI shows "Generating..." state
   - AI breaks down the todo into 3-5 actionable steps
   - Results are displayed with checkboxes for completion tracking

This approach keeps the Ollama state properly managed within Dioxus's reactive system
while maintaining a clean separation between UI logic and LLM integration.
*/