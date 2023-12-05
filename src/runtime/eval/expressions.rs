use std::{cell::RefCell, collections::HashMap, error::Error, rc::Rc};

use crate::{
    frontend::ast::{
        AssignmentExpr, BinaryExpr, CallExpr, Identifier, MemberExpr, NodeType, ObjectLiteral,
    },
    runtime::{
        environment::{Environment, EnvironmentScope},
        interpreter::evaluate,
        values::{mk_bool, mk_null, mk_number, ObjectVal, ValueType},
    },
};

pub fn eval_identifier(
    ident: &Identifier,
    env: Rc<RefCell<dyn EnvironmentScope>>,
) -> Result<ValueType, Box<dyn Error>> {
    let val = env.borrow().lookup_var(ident.symbol.clone())?;

    Ok(val)
}

pub fn eval_object_expr(
    obj: &ObjectLiteral,
    env: Rc<RefCell<dyn EnvironmentScope>>,
) -> Result<ValueType, Box<dyn Error>> {
    let mut object = ObjectVal {
        properties: HashMap::new(),
    };

    for node_type in obj.properties.clone() {
        let obj_property = match node_type {
            NodeType::Property(property) => property,
            _ => Err(format!(
                "'{:?}' in object is not of type Property",
                node_type
            ))?,
        };

        let runtime_val = match obj_property.value {
            Some(value) => evaluate(&*value, &env)?,
            None => env.borrow_mut().lookup_var(obj_property.key.clone())?,
        };

        object.properties.insert(obj_property.key, runtime_val);
    }

    Ok(ValueType::ObjectVal(object))
}

pub fn eval_call_expr(
    expr: &CallExpr,
    env: Rc<RefCell<dyn EnvironmentScope>>,
) -> Result<ValueType, Box<dyn Error>> {
    let args = {
        let mut args_vec: Vec<ValueType> = vec![];

        for arg in expr.args.clone() {
            args_vec.push(evaluate(&arg, &env)?);
        }

        args_vec
    };

    let mut function = evaluate(&expr.caller, &env)?;

    if let ValueType::NativeFnVal(native_fn) = &function {
        let result = (native_fn.call)(args);

        return Ok(result?);
    }

    if let ValueType::FunctionVal(function_val) = &mut function {
        let scope: Rc<RefCell<dyn EnvironmentScope>> =
            Environment::new(Some(function_val.declaration_env.clone()));

        for (i, value) in function_val.parameters.iter().enumerate() {
            scope.borrow().declare_var(
                value,
                args.get(i)
                    .expect("Failed to get function arg at dynamic index")
                    .clone(),
                false,
            )?;
        }

        let mut result = mk_null();

        for stmt in function_val.body.clone() {
            result = evaluate(&stmt, &scope)?;
        }

        return Ok(result);
    }

    Err(format!(
        "Cannot call value that is not a function: {:?}",
        function
    ))?
}

pub fn eval_assignment(
    node: &AssignmentExpr,
    env: Rc<RefCell<dyn EnvironmentScope>>,
) -> Result<ValueType, Box<dyn Error>> {
    if matches!(*node.assign, NodeType::MemberExpr(_)) {
        return Ok(eval_member_expr(env.clone(), Some(node), None)?);
    }

    if !matches!(*node.assign.clone(), NodeType::Identifier(_)) {
        println!("Invalid left-hand-side expression: {:?}", *node.assign);
    }

    let varname = match *node.assign.clone() {
        NodeType::Identifier(identifier) => identifier.symbol,
        _ => Err("Varname is not of type NodeType::Identifier")?,
    };

    Ok(env
        .borrow()
        .assign_var(varname, evaluate(&node.value, &env)?)?)
}

pub fn eval_binary_expr(
    binop: &BinaryExpr,
    env: Rc<RefCell<dyn EnvironmentScope>>,
) -> Result<ValueType, Box<dyn Error>> {
    let lhs = evaluate(&binop.left, &env)?;
    let rhs = evaluate(&binop.right, &env)?;

    Ok(eval_numeric_binary_expr(lhs, rhs, binop.operator.as_str())?)
}

pub fn eval_numeric_binary_expr(
    lhs: ValueType,
    rhs: ValueType,
    operator: &str,
) -> Result<ValueType, Box<dyn Error>> {
    match operator {
        "!=" => Ok(equals(lhs, rhs, false)?),
        "==" | "&&" => Ok(equals(lhs, rhs, true)?),
        "|" => {
            let llhs = match lhs {
                ValueType::BooleanVal(boolean_val) => boolean_val,
                _ => Err("llhs is not of type ValueType::BooleanVal")?,
            };
            let rrhs = match rhs {
                ValueType::BooleanVal(boolean_val) => boolean_val,
                _ => Err("rrhs is not of type ValueType::BooleanVal")?,
            };

            Ok(mk_bool(Some(llhs.value || rrhs.value)))
        }
        _ => {
            let lhs_is_number_val = matches!(lhs, ValueType::NumberVal(_));
            let rhs_is_number_val = matches!(rhs, ValueType::NumberVal(_));

            if lhs_is_number_val && rhs_is_number_val {
                let llhs = match lhs {
                    ValueType::NumberVal(boolean_val) => boolean_val,
                    _ => Err("llhs is not of type ValueType::NumberVal")?,
                };
                let rrhs = match rhs {
                    ValueType::NumberVal(boolean_val) => boolean_val,
                    _ => Err("rrhs is not of type ValueType::NumberVal")?,
                };

                return Ok(match operator {
                    "+" => mk_number(Some(llhs.value + rrhs.value)),
                    "-" => mk_number(Some(llhs.value - rrhs.value)),
                    "*" => mk_number(Some(llhs.value * rrhs.value)),
                    "/" => mk_number(Some(llhs.value / rrhs.value)),
                    "%" => mk_number(Some(llhs.value % rrhs.value)),
                    "<" => mk_bool(Some(llhs.value < rrhs.value)),
                    ">" => mk_bool(Some(llhs.value > rrhs.value)),
                    _ => Err(format!(
                        // lhs, rhs
                        "Unknown operator provided in operation.",
                    ))?,
                });
            }

            Ok(mk_null())
        }
    }
}

fn equals(lhs: ValueType, rhs: ValueType, strict: bool) -> Result<ValueType, Box<dyn Error>> {
    match lhs {
        ValueType::BooleanVal(boolean_val) => {
            let rhs_bool_val = match rhs {
                ValueType::BooleanVal(rhs_boolean_val) => rhs_boolean_val,
                _ => Err("Type of RHS does not match LHS")?,
            }
            .value;

            Ok(mk_bool(Some(if strict {
                boolean_val.value == rhs_bool_val
            } else {
                boolean_val.value != rhs_bool_val
            })))
        }
        ValueType::NumberVal(number_val) => {
            let rhs_num_val = match rhs {
                ValueType::NumberVal(rhs_number_val) => rhs_number_val,
                _ => Err("Type of RHS does not match LHS")?,
            }
            .value;

            Ok(mk_bool(Some(if strict {
                number_val.value == rhs_num_val
            } else {
                number_val.value != rhs_num_val
            })))
        }
        ValueType::StringVal(string_val) => {
            let rhs_str_val = match rhs {
                ValueType::StringVal(rhs_string_val) => rhs_string_val,
                _ => Err("Type of RHS does not match LHS")?,
            }
            .value;

            Ok(mk_bool(Some(if strict {
                string_val.value == rhs_str_val
            } else {
                string_val.value != rhs_str_val
            })))
        }
        ValueType::FunctionVal(function_val) => {
            let rhs_fn_val = match rhs {
                ValueType::FunctionVal(rhs_function_val) => rhs_function_val,
                _ => Err("Type of RHS does not match LHS")?,
            }
            .body;

            Ok(mk_bool(Some(if strict {
                function_val.body == rhs_fn_val
            } else {
                function_val.body != rhs_fn_val
            })))
        }
        ValueType::NativeFnVal(native_fn_val) => {
            let rhs_n_fn_val = match rhs {
                ValueType::NativeFnVal(rhs_native_fn_val) => rhs_native_fn_val,
                _ => Err("Type of RHS does not match LHS")?,
            }
            .call;

            Ok(mk_bool(Some(if strict {
                native_fn_val.call == rhs_n_fn_val
            } else {
                native_fn_val.call != rhs_n_fn_val
            })))
        }
        ValueType::NullVal => {
            if !matches!(rhs, ValueType::NullVal) {
                Err("Type of RHS does not match LHS")?
            }

            Ok(mk_bool(Some(true)))
        }
        #[allow(unused_variables)]
        ValueType::ObjectVal(object_val) => {
            let rhs_obj_val = match rhs {
                ValueType::ObjectVal(rhs_object_val) => rhs_object_val,
                _ => Err("Type of RHS does not match LHS")?,
            }
            .properties;

            Ok(mk_bool(Some(if strict {
                matches!(object_val.properties, rhs_obj_val)
            } else {
                !matches!(object_val.properties, rhs_obj_val)
            })))
        }
    }
}

pub fn eval_member_expr(
    env: Rc<RefCell<dyn EnvironmentScope>>,
    node: Option<&AssignmentExpr>,
    expr: Option<&MemberExpr>,
) -> Result<ValueType, Box<dyn Error>> {
    if let Some(expr_val) = expr {
        let variable = env
            .borrow_mut()
            .lookup_or_mut_object(expr_val.clone(), None, None)?;

        return Ok(variable);
    }

    if let Some(node_val) = node {
        let variable = env.borrow_mut().lookup_or_mut_object(
            match *node_val.assign.clone() {
                NodeType::MemberExpr(member_expr) => member_expr,
                _ => Err(format!(
                    "'{:?}' is not of type NodeType::MemberExpr",
                    node_val
                ))?,
            },
            Some(evaluate(&node_val.value, &env)?),
            None,
        );

        return variable;
    }

    Err(
        "Evaluating a member expression is not possible without a member or assignment expression.",
    )?
}
