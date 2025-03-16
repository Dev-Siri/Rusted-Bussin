use std::{cell::RefCell, collections::HashMap, error::Error, rc::Rc};

use crate::frontend::ast::NodeType;

use super::environment::EnvironmentScope;

#[derive(Debug, Clone)]
pub enum ValueType {
    NullVal,
    BooleanVal(BooleanVal),
    NumberVal(NumberVal),
    StringVal(StringVal),
    ObjectVal(ObjectVal),
    FunctionVal(FunctionVal),
    NativeFnVal(NativeFnVal),
}

#[derive(Debug, Clone)]
pub struct BooleanVal {
    pub value: bool,
}

#[derive(Debug, Clone)]
pub struct NumberVal {
    pub value: f32,
}

#[derive(Debug, Clone)]
pub struct StringVal {
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct ObjectVal {
    pub properties: HashMap<String, ValueType>,
}

#[derive(Debug)]
pub struct FunctionVal {
    pub name: String,
    pub parameters: Vec<String>,
    pub declaration_env: Rc<RefCell<dyn EnvironmentScope>>,
    pub body: Vec<NodeType>,
}

impl Clone for FunctionVal {
    fn clone(&self) -> Self {
        Self {
            body: self.body.clone(),
            declaration_env: self.declaration_env.clone(),
            name: self.name.clone(),
            parameters: self.parameters.clone(),
        }
    }
}

pub type FunctionCall = fn(args: Vec<ValueType>) -> Result<ValueType, Box<dyn Error>>;

#[derive(Debug, Clone)]
pub struct NativeFnVal {
    pub name: String,
    pub call: FunctionCall,
}

pub fn mk_native_fn(name: &str, call: FunctionCall) -> ValueType {
    ValueType::NativeFnVal(NativeFnVal {
        name: name.to_string(),
        call,
    })
}

pub fn mk_number(n: Option<f32>) -> ValueType {
    ValueType::NumberVal(NumberVal {
        value: n.unwrap_or_default(),
    })
}

pub fn mk_null() -> ValueType {
    ValueType::NullVal
}

pub fn mk_bool(b: Option<bool>) -> ValueType {
    ValueType::BooleanVal(BooleanVal {
        value: b.unwrap_or(true),
    })
}

pub fn mk_string(value: String) -> ValueType {
    ValueType::StringVal(StringVal { value })
}

pub fn mk_object(obj: HashMap<String, ValueType>) -> ValueType {
    ValueType::ObjectVal(ObjectVal { properties: obj })
}
