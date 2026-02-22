use async_openai::{Client, config::OpenAIConfig};
use clap::Parser;
use serde_json::{Value, json};
use std::{env, process};

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(short = 'p', long)]
    prompt: String,
}

struct QueryResult {
    messages: Vec<Value>,
    is_done: bool,
}

// read file fn
async fn read_file(file_path: String) -> Result<String, std::io::Error> {
    let contents = tokio::fs::read_to_string(file_path).await?;
    Ok(contents)
}

async fn query_ai(
    client: &Client<OpenAIConfig>,
    mut messages: Vec<Value>,
) -> Result<QueryResult, Box<dyn std::error::Error>> {
    let response: Value = client
        .chat()
        .create_byot(json!({
        "messages": messages,
        "model": "anthropic/claude-haiku-4.5",
        "tools": [{
          "type": "function",
          "function": {
            "name": "read_file",
            "description": "Read and return the contents of a file",
            "parameters": {
              "type": "object",
              "properties": {
                "file_path": {
                  "type": "string",
                  "description": "The path to the file to read"
                }
              },
              "required": ["file_path"]
            }
          }
        }]
                }))
        .await?;
    messages.push(response["choices"][0]["message"].clone());

    // TODO: Uncomment the lines below to pass the first stage
    if let Some(content) = response["choices"][0]["finish_reason"].as_str() {
        if content == "tool_calls" {
            let tool_call_specs = &response["choices"][0]["message"]["tool_calls"];
            if !tool_call_specs.is_null() {
                let args: Result<Value, _> = serde_json::from_str(
                    tool_call_specs[0]["function"]["arguments"]
                        .as_str()
                        .unwrap_or_default(),
                );
                // parse json string and get file_path
                let mut file_path: String = "cat".to_string();
                if let Ok(args_value) = args {
                    file_path = args_value["file_path"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string();
                }
                let fn_name = tool_call_specs[0]["function"]["name"]
                    .as_str()
                    .unwrap_or_default();

                // call fn name with args
                if fn_name == "read_file" {
                    let file_contents = read_file(file_path).await?;
                    // append to message_array
                    messages.push(json!({"role":"tool", "tool_call_id": tool_call_specs[0]["id"], "content":file_contents}));
                    return Ok(QueryResult {
                        messages,
                        is_done: false,
                    });
                } else {
                    return Err("Unknown function name".into());
                }
            } else {
                return Err(format!("No tool calls found").into());
            }
            // return Ok(());
        }
    }

    // If not tool call, just print response
    if let Some(content) = response["choices"][0]["message"]["content"].as_str() {
        println!("{}", content);
        messages.push(json!({"role":"assistant", "content":content}));
        return Ok(QueryResult {
            messages,
            is_done: true,
        });
    } else {
        return Err("No content found".into());
    }
}

fn setup() -> OpenAIConfig {
    let base_url = env::var("OPENROUTER_BASE_URL")
        .unwrap_or_else(|_| "https://openrouter.ai/api/v1".to_string());

    let api_key = env::var("OPENROUTER_API_KEY").unwrap_or_else(|_| {
        eprintln!("OPENROUTER_API_KEY is not set");
        process::exit(1);
    });

    let config = OpenAIConfig::new()
        .with_api_base(base_url)
        .with_api_key(api_key);
    return config;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let config = setup();

    let client: Client<OpenAIConfig> = Client::with_config(config);

    let mut message_array = vec![json!({ "role": "user", "content": args.prompt })];

    for _i in 0..5 {
        let result = query_ai(&client, message_array).await?;
        message_array = result.messages;
        // if last message in array has finish_reason of Stop, print that and exit

        if result.is_done {
            break;
        }
    }
    Ok(())
}
