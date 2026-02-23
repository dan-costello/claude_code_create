mod builder;
mod tools;

use async_openai::{Client, config::OpenAIConfig};
use builder::{Tool, ToolBuilder};
use clap::Parser;
use serde_json::{Value, json};
use std::{env, process};
use tools::{execute_bash, read_file, write_file};

// TODO:
// finish_reason match isn't exhaustive yet

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

struct ToolCallResult {
    output: String,
    id: String,
}

async fn call_ai(
    client: &Client<OpenAIConfig>,
    messages: &[Value],
    tools: &[Tool],
) -> Result<Value, Box<dyn std::error::Error>> {
    let response: Value = client
        .chat()
        .create_byot(json!({
            "messages": messages,
            "model": "anthropic/claude-haiku-4.5",
            "tools": tools.iter().map(|t| serde_json::to_value(t)).collect::<Result<Vec<_>, _>>()?
        }))
        .await?;
    Ok(response)
}

async fn dispatch_tool(tool_call: &Value) -> Result<ToolCallResult, Box<dyn std::error::Error>> {
    let fn_name = tool_call["function"]["name"].as_str().unwrap_or_default();
    let fn_args: Result<Value, _> = serde_json::from_str(
        tool_call["function"]["arguments"]
            .as_str()
            .unwrap_or_default(),
    );
    if fn_name == "read_file" {
        // parse json string and get file_path
        if let Ok(args_value) = fn_args {
            let file_path = args_value["file_path"]
                .as_str()
                .ok_or("missing file_path argument")?
                .to_string();
            let file_contents = read_file(file_path).await?;
            return Ok(ToolCallResult {
                output: file_contents,
                id: tool_call["id"].as_str().unwrap_or_default().to_string(),
            });
        } else {
            return Err("Failed to parse function arguments".into());
        }
    } else if fn_name == "write_file" {
        if let Ok(args_value) = fn_args {
            let file_path = args_value["file_path"]
                .as_str()
                .ok_or("missing file_path argument")?
                .to_string();
            let content = args_value["content"]
                .as_str()
                .ok_or("missing file_path argument")?
                .to_string();
            let _ = write_file(file_path, content).await?;
            return Ok(ToolCallResult {
                output: "Content written to file".to_string(),
                id: tool_call["id"].as_str().unwrap_or_default().to_string(),
            });
        } else {
            return Err("Failed to parse function arguments".into());
        }
    } else if fn_name == "Bash" {
        if let Ok(args_value) = fn_args {
            let command = args_value["command"]
                .as_str()
                .ok_or("missing file_path argument")?
                .to_string();
            let output = execute_bash(command).await?;
            return Ok(ToolCallResult {
                output,
                id: tool_call["id"].as_str().unwrap_or_default().to_string(),
            });
        } else {
            return Err("Failed to parse function arguments".into());
        }
    } else {
        return Err("Unknown function name".into());
    }
}

async fn query_ai(
    client: &Client<OpenAIConfig>,
    mut messages: Vec<Value>,
    tools: &[Tool],
) -> Result<QueryResult, Box<dyn std::error::Error>> {
    // println!("Sending messages: {:?}", messages);
    let response: Value = call_ai(client, &messages, tools).await?;
    // println!("AI response: {}", response);
    messages.push(response["choices"][0]["message"].clone());

    if let Some(content) = response["choices"][0]["finish_reason"].as_str() {
        if content == "tool_calls" {
            let tool_call_specs = &response["choices"][0]["message"]["tool_calls"];
            // println!("{}", tool_call_specs);
            if !tool_call_specs.is_null() {
                let tool_result = dispatch_tool(&tool_call_specs[0]).await?;

                messages.push(json!({"role":"tool", "tool_call_id": tool_result.id, "content":tool_result.output}));
                return Ok(QueryResult {
                    messages,
                    is_done: false,
                });
            } else {
                return Err(format!("No tool calls found").into());
            }
        }
    }

    // If not tool call, just print response
    if let Some(content) = response["choices"][0]["message"]["content"].as_str() {
        println!("{}", content);
        return Ok(QueryResult {
            messages,
            is_done: true,
        });
    } else {
        return Err("No content found".into());
    }
}

fn setup() -> Client<OpenAIConfig> {
    let base_url = env::var("OPENROUTER_BASE_URL")
        .unwrap_or_else(|_| "https://openrouter.ai/api/v1".to_string());

    let api_key = env::var("OPENROUTER_API_KEY").unwrap_or_else(|_| {
        eprintln!("OPENROUTER_API_KEY is not set");
        process::exit(1);
    });

    let config = OpenAIConfig::new()
        .with_api_base(base_url)
        .with_api_key(api_key);

    let client = Client::with_config(config);
    return client;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup OpenAI client
    let client = setup();

    // parse args to get inital query
    let args = Args::parse();
    let mut message_array = vec![json!({ "role": "user", "content": args.prompt })];

    // init the tools we need
    let read_file_tool = ToolBuilder::new("read_file", "Read and return the contents of a file")
        .param::<String>("file_path", "The path to the file to read")
        .build();
    let write_file_tool = ToolBuilder::new("write_file", "Write content to a file")
        .param::<String>("file_path", "The path to the file to read")
        .param::<String>("content", "The content to write to the file")
        .build();
    let execute_bash_tool = ToolBuilder::new("Bash", "Execute a shell command")
        .param::<String>("command", "The command to execute")
        .build();

    // Loop until we get a text response(not tool call)
    loop {
        let result = query_ai(
            &client,
            message_array,
            &[read_file_tool.clone(), write_file_tool.clone(), execute_bash_tool.clone()],
        )
        .await?;
        message_array = result.messages;
        if result.is_done {
            break;
        }
    }
    Ok(())
}
