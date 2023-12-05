use std::{cell::RefCell, error::Error, rc::Rc};

use crate::frontend::ast::NodeType;

use super::{
    environment::EnvironmentScope,
    eval::{
        expressions::{
            eval_assignment, eval_binary_expr, eval_call_expr, eval_identifier, eval_member_expr,
            eval_object_expr,
        },
        statements::{
            eval_for_statement, eval_function_declaration, eval_if_statement, eval_program,
            eval_try_catch_statement, eval_val_declaration,
        },
    },
    values::{NumberVal, StringVal, ValueType},
};

pub fn evaluate<'a>(
    ast_node: &NodeType,
    env: &Rc<RefCell<dyn EnvironmentScope>>,
) -> Result<ValueType, Box<dyn Error>> {
    match ast_node {
        NodeType::NumericLiteral(numeric_literal) => Ok(ValueType::NumberVal(NumberVal {
            value: numeric_literal.value,
        })),
        NodeType::StringLiteral(string_literal) => Ok(ValueType::StringVal(StringVal {
            value: string_literal.value.clone(),
        })),
        NodeType::Identifier(identifier) => eval_identifier(identifier, env.clone()),
        NodeType::ObjectLiteral(object_literal) => eval_object_expr(object_literal, env.clone()),
        NodeType::CallExpr(call_expr) => eval_call_expr(call_expr, env.clone()),
        NodeType::AssignmentExpr(assignment_expr) => eval_assignment(assignment_expr, env.clone()),
        NodeType::BinaryExpr(binary_expr) => eval_binary_expr(binary_expr, env.clone()),
        NodeType::Program(program) => eval_program(program, env.clone()),
        NodeType::IfStatement(if_statement) => eval_if_statement(if_statement, env.clone()),
        NodeType::ForStatement(for_statement) => eval_for_statement(for_statement, env.clone()),
        NodeType::MemberExpr(member_expr) => eval_member_expr(env.clone(), None, Some(member_expr)),
        NodeType::TryCatchStatement(try_catch_statement) => {
            eval_try_catch_statement(try_catch_statement, env.clone())
        }
        NodeType::VarDeclaration(var_declaration) => {
            eval_val_declaration(var_declaration, env.clone())
        }
        NodeType::FunctionDeclaration(function_declaration) => {
            eval_function_declaration(function_declaration, env.clone())
        }
        _ => Err(format!(
            "This AST node has not yet been setup for interpretation {:?}",
            ast_node
        ))?,
    }
}
