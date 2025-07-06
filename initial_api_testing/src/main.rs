use ollama_rs::{
    Ollama, generation::{
        completion::request::GenerationRequest,
        parameters::FormatType
    }
};
use tokio_stream::StreamExt;
use serde::{Deserialize, Serialize};

const JSON_FORMAT: &str = r#"{
    "steps": [
        {
            "content": "<step>",
        }
    ]
}"#;

fn clean_json_string(input: &str) -> String {
    input
        .trim()  // Trims whitespace
        .lines()  // Iterate over all lines
        .map(|line| line.trim())  // Trim each line
        .filter(|line| !line.is_empty())  // Filter out empty lines
        .collect::<Vec<_>>()  // Collect all of them into a vector
        .join("")  // Join the final result into one string
}

#[derive(Debug, Serialize, Deserialize)]
struct Task {
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Todo {
    steps: Vec<Task>,
}


impl Todo {
    fn from_string(s: String) -> Self {
        let mut steps = vec![];
        let cleaned_data = clean_json_string(&s);
        match serde_json::from_str::<Todo>(&cleaned_data) {
            Ok(t) => for task in t.steps {
                steps.push(Task { content: task.content });
            },
            Err(e) => eprintln!("Error parsing JSON: {}", e),
        }
        Todo {steps}
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let ollama = Ollama::default();

    let model = "gemma3n".to_string();
    let control = format!(
        "You are an assistant who breaks up todo items into achievable tasks.
        The resulting items must be no more than 5.
        Each item must a short and concise todo entry.
        The result must be json serialisable.
        The result must strictly match this format, including step number indexes: {}", JSON_FORMAT
    );
    let prompt = format!("{}.  The todo is: Prepare documentation for a new plugin for Blender.", control);

    let mut resp = ollama.generate_stream(GenerationRequest::new(model, prompt).format(FormatType::Json)).await.unwrap();

    let mut raw_data: Vec<String> = vec![];
    while let Some(res) = resp.next().await {
        let responses = res.unwrap();
        for r in responses {
            raw_data.push(r.response);
        }
    }

    let new_list = Todo::from_string(raw_data.join(""));

    println!("{:?}", new_list);

    Ok(())
}
