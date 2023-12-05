use std::{cell::RefCell, error::Error, rc::Rc};

use crate::{
    frontend::ast::{
        ForStatement, FunctionDeclaration, IfStatement, NodeType, Program, TryCatchStatement,
        VarDeclaration,
    },
    runtime::{
        environment::{Environment, EnvironmentScope},
        interpreter::evaluate,
        values::{mk_null, mk_string, FunctionVal, ValueType},
    },
};

use super::expressions::eval_assignment;

pub fn eval_program(
    program: &Program,
    env: Rc<RefCell<dyn EnvironmentScope>>,
) -> Result<ValueType, Box<dyn Error>> {
    let mut last_evaluated = mk_null();

    for statement in &program.body {
        last_evaluated = evaluate(statement, &env)?;
    }

    Ok(last_evaluated)
}

pub fn eval_if_statement(
    declaration: &IfStatement,
    env: Rc<RefCell<dyn EnvironmentScope>>,
) -> Result<ValueType, Box<dyn Error>> {
    let test = evaluate(&declaration.test, &env)?;
    let test_is_bool = match test {
        ValueType::BooleanVal(boolean_val) => boolean_val.value,
        _ => Err("Boolean value for if statement is not a boolean expression")?,
    };

    Ok(if test_is_bool {
        eval_body(&declaration.body, env, true)?
    } else if let Some(alternate) = &declaration.alternate {
        eval_body(alternate, env, true)?
    } else {
        mk_null()
    })
}

fn eval_body(
    body: &Vec<NodeType>,
    env: Rc<RefCell<dyn EnvironmentScope>>,
    new_env: bool,
) -> Result<ValueType, Box<dyn Error>> {
    let scope = if new_env {
        let binding: Rc<RefCell<dyn EnvironmentScope>> = Environment::new(Some(env.clone()));
        binding
    } else {
        env
    };

    let mut result = mk_null();

    for stmt in body {
        result = evaluate(stmt, &scope)?;
    }

    Ok(result)
}

pub fn eval_val_declaration(
    declaration: &VarDeclaration,
    env: Rc<RefCell<dyn EnvironmentScope>>,
) -> Result<ValueType, Box<dyn Error>> {
    let value = if let Some(declaration_value) = &declaration.value {
        evaluate(declaration_value, &env)?
    } else {
        mk_null()
    };

    Ok(env
        .borrow()
        .declare_var(&declaration.identifier, value, declaration.constant)?)
}

pub fn eval_for_statement(
    declaration: &ForStatement,
    env: Rc<RefCell<dyn EnvironmentScope>>,
) -> Result<ValueType, Box<dyn Error>> {
    let n_env: Rc<RefCell<dyn EnvironmentScope>> = Environment::new(Some(env.clone()));

    eval_val_declaration(
        &if let NodeType::VarDeclaration(var_declaration) = &*declaration.init {
            var_declaration.clone()
        } else {
            Err(format!(
                "'{:?}' is not of type NodeType::VarDeclaration",
                declaration.init
            ))?
        },
        n_env.clone(),
    )?;

    let body = declaration.body.clone();
    let update = match *declaration.update.clone() {
        NodeType::AssignmentExpr(assignment_expr) => assignment_expr,
        _ => Err("update of for() is not of type NodeType::AssignmentExpr")?,
    };

    let test_vtype = evaluate(&*declaration.test, &n_env)?;
    let mut test = match &test_vtype {
        ValueType::BooleanVal(boolean_val) => boolean_val,
        _ => Err("Test condition of for() is not of type ValueType::BooleanVal")?,
    };

    if !test.value {
        return Ok(mk_null());
    }

    loop {
        eval_assignment(&update, n_env.clone())?;

        let body_to_eval: Rc<RefCell<dyn EnvironmentScope>> = Environment::new(Some(n_env.clone()));

        eval_body(&body, body_to_eval, false)?;

        let test_loop_vtype = evaluate(&*declaration.test, &n_env)?;
        test = match &test_loop_vtype {
            ValueType::BooleanVal(boolean_val) => boolean_val,
            _ => Err("Test condition of for() is not of type ValueType::BooleanVal")?,
        };

        if !test.value {
            break;
        }
    }

    Ok(mk_null())
}

pub fn eval_try_catch_statement(
    declaration: &TryCatchStatement,
    env: Rc<RefCell<dyn EnvironmentScope>>,
) -> Result<ValueType, Box<dyn Error>> {
    let try_env: Rc<RefCell<dyn EnvironmentScope>> = Environment::new(Some(env.clone()));

    Ok(match eval_body(&declaration.body, try_env, false) {
        Ok(body) => body,
        Err(err) => {
            let catch_env: Rc<RefCell<dyn EnvironmentScope>> = Environment::new(Some(env.clone()));
            env.borrow()
                .assign_var("error".to_string(), mk_string(err.to_string()))?;
            eval_body(&declaration.alternate, catch_env, false)?
        }
    })
}

pub fn eval_function_declaration(
    declaration: &FunctionDeclaration,
    env: Rc<RefCell<dyn EnvironmentScope>>,
) -> Result<ValueType, Box<dyn Error>> {
    let function = ValueType::FunctionVal(FunctionVal {
        name: declaration.name.clone(),
        parameters: declaration.parameters.clone(),
        declaration_env: env.clone(),
        body: declaration.body.clone(),
    });

    return env
        .borrow()
        .declare_var(declaration.name.as_str(), function, true);
}
