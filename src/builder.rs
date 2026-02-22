use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Clone)]
pub struct ToolParameter {
    #[serde(rename = "type")]
    kind: String,
    description: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ToolFunctionParams {
    #[serde(rename = "type")]
    kind: String, //
    properties: HashMap<String, ToolParameter>,
    required: Vec<String>,
}

impl ToolFunctionParams {
    fn _new(properties: HashMap<String, ToolParameter>, required: Vec<String>) -> Self {
        ToolFunctionParams {
            kind: "object".to_string(),
            properties,
            required,
        }
    }
}
#[derive(Deserialize, Serialize, Clone)]
pub struct ToolFunction {
    name: String,
    description: String,
    parameters: ToolFunctionParams,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Tool {
    #[serde(rename = "type")]
    kind: &'static str,
    function: ToolFunction,
}

impl Tool {
    pub fn new(function: ToolFunction) -> Self {
        Tool {
            kind: "function",
            function,
        }
    }
}

#[async_trait::async_trait]
pub trait ToolImpl {
    fn name(&self) -> &'static str;
    fn definition(&self) -> Tool;
    async fn execute(&self, args: Value) -> Result<String, Box<dyn std::error::Error>>;
}

pub trait JsonType {
    fn json_type() -> String;
}

impl JsonType for String {
    fn json_type() -> String {
        "string".to_string()
    }
}
impl JsonType for i64 {
    fn json_type() -> String {
        "integer".to_string()
    }
}
impl JsonType for f64 {
    fn json_type() -> String {
        "number".to_string()
    }
}
impl JsonType for bool {
    fn json_type() -> String {
        "boolean".to_string()
    }
}

pub struct ToolBuilder {
    name: String,
    description: String,
    properties: HashMap<String, ToolParameter>,
    required: Vec<String>,
}


pub struct ToolRegistry(Vec<Box<dyn ToolImpl>>);

impl ToolRegistry {
    pub fn new(tools: Vec<Box<dyn ToolImpl>>) -> Self { 
        ToolRegistry(tools) 
    }
    
    pub fn definitions(&self) -> Vec<Tool> { 
        self.0.iter().map(|t| t.definition()).collect() 
    }
    
    pub async fn dispatch(&self, name: &str, args: Value) -> Result<String, Box<dyn std::error::Error>> {
        let tool = self.0.iter()
            .find(|t| t.name() == name)
            .ok_or_else(|| format!("Unknown tool: {}", name))?;
        
        tool.execute(args).await
    }
}

impl ToolBuilder {
    pub fn new(name: &str, description: &str) -> Self {
        ToolBuilder {
            name: name.to_string(),
            description: description.to_string(),
            properties: HashMap::new(),
            required: Vec::new(),
        }
    }

    pub fn param<T: JsonType>(mut self, name: &str, description: &str) -> Self {
        self.properties.insert(
            name.to_string(),
            ToolParameter {
                kind: T::json_type(),
                description: description.to_string(),
            },
        );
        self.required.push(name.to_string());
        self
    }

    pub fn _optional_param<T: JsonType>(mut self, name: &str, description: &str) -> Self {
        self.properties.insert(
            name.to_string(),
            ToolParameter {
                kind: T::json_type(),
                description: description.to_string(),
            },
        );
        self // not pushed to required
    }

    pub fn build(self) -> Tool {
        Tool::new(ToolFunction {
            name: self.name,
            description: self.description,
            parameters: ToolFunctionParams {
                kind: "object".to_string(),
                properties: self.properties,
                required: self.required,
            },
        })
    }
}
