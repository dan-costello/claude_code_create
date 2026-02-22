
use serde::{Deserialize, Serialize};
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
    fn new(properties: HashMap<String, ToolParameter>, required: Vec<String>) -> Self {
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

    pub fn optional_param<T: JsonType>(mut self, name: &str, description: &str) -> Self {
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
