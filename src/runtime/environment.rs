// I have no idea what the hell is going on in this file
use std::{
    cell::RefCell,
    collections::HashMap,
    error::Error,
    f32::consts,
    fmt::{Debug, Formatter},
    rc::Rc,
};

use crate::frontend::ast::{Identifier, MemberExpr, NodeType};

use super::{
    eval::native_fns::{
        exec, format, input, math_abs, math_ceil, math_random, math_round, math_sqrt, print_values,
        strcon, time_function,
    },
    values::{mk_bool, mk_native_fn, mk_null, mk_number, mk_object, ObjectVal, ValueType},
};

pub fn create_global_env() -> Result<Rc<RefCell<dyn EnvironmentScope>>, Box<dyn Error>> {
    let env: Rc<RefCell<dyn EnvironmentScope>> = Environment::new(None);

    env.borrow()
        .declare_var("true", mk_bool(Some(true)), true)?;
    env.borrow()
        .declare_var("false", mk_bool(Some(false)), true)?;
    env.borrow().declare_var("null", mk_null(), true)?;

    env.borrow().declare_var("error", mk_null(), false)?;

    env.borrow()
        .declare_var("println", mk_native_fn("println", print_values), true)?;
    env.borrow()
        .declare_var("exec", mk_native_fn("exec", exec), true)?;
    env.borrow()
        .declare_var("input", mk_native_fn("input", input), true)?;

    let mut math: HashMap<String, ValueType> = HashMap::new();

    math.insert("pi".to_string(), mk_number(Some(consts::PI)));
    math.insert("sqrt".to_string(), mk_native_fn("math.sqrt", math_sqrt));
    math.insert(
        "random".to_string(),
        mk_native_fn("math.random", math_random),
    );
    math.insert("round".to_string(), mk_native_fn("math.round", math_round));
    math.insert("ceil".to_string(), mk_native_fn("math.ceil", math_ceil));
    math.insert("abs".to_string(), mk_native_fn("math.abs", math_abs));

    env.borrow().declare_var("math", mk_object(math), true)?;
    env.borrow()
        .declare_var("strcon", mk_native_fn("strcon", strcon), true)?;
    env.borrow()
        .declare_var("format", mk_native_fn("format", format), true)?;
    env.borrow()
        .declare_var("time", mk_native_fn("time", time_function), true)?;

    Ok(env)
}

#[derive(Debug)]
pub struct Environment {
    parent: Option<Rc<RefCell<dyn EnvironmentScope>>>,
    variables: RefCell<HashMap<String, ValueType>>,
    constants: RefCell<Vec<String>>,
}

pub trait EnvironmentScope {
    fn declare_var(
        &self,
        varname: &str,
        value: ValueType,
        constant: bool,
    ) -> Result<ValueType, Box<dyn Error>>;
    fn assign_var(&self, varname: String, value: ValueType) -> Result<ValueType, Box<dyn Error>>;
    fn lookup_or_mut_object(
        &self,
        expr: MemberExpr,
        value: Option<ValueType>,
        property: Option<Identifier>,
    ) -> Result<ValueType, Box<dyn Error>>;
    fn lookup_var(&self, varname: String) -> Result<ValueType, Box<dyn Error>>;
    fn resolve(&self, varname: String) -> Result<Rc<RefCell<Environment>>, Box<dyn Error>>;
}

impl Environment {
    pub(crate) fn new(parent_env: Option<Rc<RefCell<dyn EnvironmentScope>>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            parent: parent_env,
            variables: RefCell::new(HashMap::new()),
            constants: RefCell::new(vec![]),
        }))
    }
}

impl Debug for dyn EnvironmentScope {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "EnvironmentScope")
    }
}

impl Clone for Environment {
    fn clone(&self) -> Self {
        Self {
            parent: self.parent.clone(),
            variables: self.variables.clone(),
            constants: self.constants.clone(),
        }
    }
}

impl EnvironmentScope for Environment {
    fn declare_var(
        &self,
        varname: &str,
        value: ValueType,
        constant: bool,
    ) -> Result<ValueType, Box<dyn Error>> {
        if self.variables.borrow().contains_key(varname.clone()) {
            Err(format!(
                "Cannot declare variable {}. As it already is defined.",
                varname
            ))?
        }

        self.variables
            .borrow_mut()
            .insert(varname.clone().to_string(), value.clone());

        if constant {
            self.constants
                .borrow_mut()
                .push(varname.clone().to_string());
        }

        Ok(value)
    }

    fn assign_var(&self, varname: String, value: ValueType) -> Result<ValueType, Box<dyn Error>> {
        let env = self.resolve(varname.clone())?;

        if env.borrow().constants.borrow().contains(&varname) {
            Err(format!(
                "Cannot reassign to variable '{}' as it's constant",
                varname
            ))?
        }

        self.variables
            .borrow_mut()
            .insert(varname.clone(), value.clone());

        Ok(value)
    }

    fn lookup_or_mut_object(
        &self,
        expr: MemberExpr,
        value: Option<ValueType>,
        property: Option<Identifier>,
    ) -> Result<ValueType, Box<dyn Error>> {
        if let NodeType::MemberExpr(inner_expr) = *expr.object.clone() {
            return self.lookup_or_mut_object(
                inner_expr,
                value,
                if let NodeType::Identifier(identifier) = *expr.property.clone() {
                    Some(identifier)
                } else {
                    Err(format!("`{:?}` is not an Identifier", expr.object))?
                },
            );
        }

        let varname = if let NodeType::Identifier(identifier) = *expr.object.clone() {
            identifier.symbol.clone()
        } else {
            Err(format!("'{:?}' is not an Identifier", expr.object))?
        };

        let env = Rc::clone(&self.resolve(varname.clone())?);
        let mut binding = env.borrow_mut();
        let mut past_val: &mut ObjectVal = match binding
            .variables
            .get_mut()
            .get_mut(&varname)
            .expect(format!("Variable '{}' doesn't exist", varname).as_str())
        {
            ValueType::ObjectVal(object_val) => object_val,
            _ => Err(format!("'{}' is not an ObjectVal", varname))?,
        };

        let prop = if let Some(identifier) = property {
            identifier.symbol
        } else {
            match *expr.property.clone() {
                NodeType::Identifier(identifier) => identifier.symbol,
                _ => Err(format!("'{:?}' is not an Identifier", expr.object))?,
            }
        };

        let current_prop = match *expr.property.clone() {
            NodeType::Identifier(identifier) => Some(identifier.symbol),
            _ => None,
        };

        if let Some(defined_value) = value {
            past_val.properties.insert(prop, defined_value);
        }

        if let Some(defined_current_prop) = current_prop.clone() {
            past_val = match past_val
                .properties
                .get_mut(&defined_current_prop)
                .expect("&current_prop doesn't exist.")
            {
                ValueType::ObjectVal(object_val) => object_val,
                _ => Err(format!(
                    "'{}' is not an ObjectVal",
                    current_prop.unwrap_or("undefined".to_string())
                ))?,
            };
        }

        Ok(ValueType::ObjectVal(past_val.clone()))
    }

    fn lookup_var(&self, varname: String) -> Result<ValueType, Box<dyn Error>> {
        let env = Rc::clone(&self.resolve(varname.clone())?);
        let mut binding = env.borrow_mut();

        Ok(binding
            .variables
            .get_mut()
            .get(&varname)
            .expect(format!("'{}' does not exist", varname).as_str())
            .clone())
    }

    fn resolve(&self, varname: String) -> Result<Rc<RefCell<Environment>>, Box<dyn Error>> {
        if self.variables.borrow().contains_key(&varname) {
            return Ok(Rc::new(RefCell::new(self.clone())));
        }

        match &self.parent {
            Some(parent) => {
                let env = parent.borrow().resolve(varname)?;
                Ok(env)
            }
            None => Err(format!(
                "Cannot resolve '{}' as it does not exist.",
                varname
            ))?,
        }
    }
}
