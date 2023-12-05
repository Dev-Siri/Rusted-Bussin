use std::process;

use super::{
    ast::{
        AssignmentExpr, BinaryExpr, CallExpr, ForStatement, FunctionDeclaration, Identifier,
        IfStatement, MemberExpr, NodeType, NumericLiteral, ObjectLiteral, Program, Property,
        StringLiteral, TryCatchStatement, VarDeclaration,
    },
    lexer::{tokenize, Token, TokenType},
};

pub struct Parser {
    tokens: Vec<Token>,
}

pub trait Parse {
    fn new() -> Self;
    fn create_ast(&mut self, input: String) -> NodeType;
}

impl Parse for Parser {
    fn new() -> Self {
        Parser { tokens: vec![] }
    }

    fn create_ast(&mut self, input: String) -> NodeType {
        self.tokens = tokenize(input);

        let mut program = Program { body: vec![] };

        while self.not_eof() {
            program.body.push(self.parse_stmt());
        }

        NodeType::Program(program)
    }
}

impl Parser {
    fn not_eof(&self) -> bool {
        self.tokens
            .get(0)
            .expect("Failed to get token at pos 0")
            .tok_type
            != TokenType::EOF
    }

    fn at(&self) -> &Token {
        self.tokens.get(0).expect("Failed to get token at pos 0")
    }

    fn eat(&mut self) -> Token {
        self.tokens.remove(0)
    }

    // For some reason, rust-analyzer thinks tok_type is unused here, which it definitely isn't
    fn expect(&mut self, tok_type: TokenType, err: &str) -> Token {
        let prev = self.eat();

        if prev.tok_type.clone() != tok_type {
            println!("Parser error:\n {} {:?}", err, prev.tok_type);
            process::exit(1);
        }

        prev
    }

    fn parse_stmt(&mut self) -> NodeType {
        match self.at().tok_type {
            TokenType::Let | TokenType::Const => self.parse_var_declaration(),
            TokenType::Fn => self.parse_function_declaration(),
            TokenType::If => self.parse_if_statement(),
            TokenType::For => self.parse_for_statement(),
            _ => self.parse_expr(),
        }
    }

    fn parse_block_statement(&mut self) -> Vec<NodeType> {
        self.expect(
            TokenType::OpenBrace,
            "Opening brace (\"{\") expected while parsing code block.",
        );

        let mut body: Vec<NodeType> = vec![];

        while self.not_eof() && !matches!(self.at().tok_type, TokenType::CloseBrace) {
            let stmt = self.parse_stmt();
            body.push(stmt);
        }

        self.expect(
            TokenType::CloseBrace,
            "Closing brace (\"}\") expected while parsing code block.",
        );

        body
    }

    fn parse_for_statement(&mut self) -> NodeType {
        self.eat();
        self.expect(
            TokenType::OpenParen,
            "Opening parenthesis (\"(\") expected following \"for\" statement.",
        );

        let init = self.parse_var_declaration();
        let test = self.parse_expr();

        self.expect(
            TokenType::Semicolon,
            "Semicolon (\";\") expected following \"test expression\" in \"for\" statement.",
        );

        let update = self.parse_assignment_expr();

        self.expect(TokenType::CloseParen, "Closing parenthesis (\"(\") expected following \"additive expression\" in \"for\" statement.");

        let body = self.parse_block_statement();

        NodeType::ForStatement(ForStatement {
            body,
            init: Box::new(init),
            test: Box::new(test),
            update: Box::new(update),
        })
    }

    fn parse_if_statement(&mut self) -> NodeType {
        self.eat();
        self.expect(
            TokenType::OpenParen,
            "Opening parenthesis (\"(\") expected following \"if\" statement.",
        );

        let test = self.parse_expr();

        self.expect(
            TokenType::CloseParen,
            "Closing parenthesis (\"(\") expected following \"if\" statement.",
        );

        let body = self.parse_block_statement();

        let mut alternate: Vec<NodeType> = vec![];

        if matches!(self.at().tok_type, TokenType::Else) {
            self.eat();

            if matches!(self.at().tok_type, TokenType::If) {
                alternate = vec![self.parse_if_statement()];
            } else {
                alternate = self.parse_block_statement();
            }
        }

        NodeType::IfStatement(IfStatement {
            body,
            test: Box::new(test),
            alternate: Some(alternate),
        })
    }

    fn parse_function_declaration(&mut self) -> NodeType {
        self.eat();
        let name = self
            .expect(
                TokenType::Identifier,
                "Function name expected following \"fn\" statement.",
            )
            .value;

        let args = self.parse_args();
        let mut params: Vec<String> = vec![];

        for arg in args.iter() {
            if let NodeType::Identifier(identifier) = arg {
                params.push(identifier.symbol.clone());
            } else {
                println!("Arguments for \"fn\" statement must be of type \"String\".");
                process::exit(1);
            }
        }

        let body = self.parse_block_statement();

        NodeType::FunctionDeclaration(FunctionDeclaration {
            body,
            name,
            parameters: params,
        })
    }

    fn parse_var_declaration(&mut self) -> NodeType {
        let is_constant = matches!(self.eat().tok_type, TokenType::Const);
        let identifier = self
            .expect(
                TokenType::Identifier,
                "Variable name expected following \"let\"/\"const\" statement",
            )
            .value;

        if matches!(self.at().tok_type, TokenType::Semicolon) {
            self.eat();

            if is_constant {
                println!("Constant variables must have assigned values.");
                process::exit(1);
            }

            return NodeType::VarDeclaration(VarDeclaration {
                constant: false,
                identifier,
                value: None,
            });
        }

        self.expect(TokenType::Equals, "Equals (\"=\") expected following \"identifier\" declaration in \"let\"/\"const\" statement.");

        let declaration = NodeType::VarDeclaration(VarDeclaration {
            constant: is_constant,
            identifier,
            value: Some(Box::new(self.parse_expr())),
        });

        if matches!(self.at().tok_type, TokenType::String) {
            self.eat();
        }

        self.expect(
            TokenType::Semicolon,
            "Semicolon (\";\") expected at the end of \"let\"/\"const\" statement.",
        );

        declaration
    }

    fn parse_expr(&mut self) -> NodeType {
        self.parse_assignment_expr()
    }

    fn parse_assignment_expr(&mut self) -> NodeType {
        let left = self.parse_object_expr();

        if matches!(self.at().tok_type, TokenType::Equals) {
            self.eat();
            let value = self.parse_assignment_expr();

            return NodeType::AssignmentExpr(AssignmentExpr {
                assign: Box::new(left),
                value: Box::new(value),
            });
        }

        left
    }

    fn parse_args(&mut self) -> Vec<NodeType> {
        self.expect(
            TokenType::OpenParen,
            "Opening parenthesis (\"(\") expected while parsing arguments.",
        );
        let args: Vec<NodeType> = if matches!(self.at().tok_type, TokenType::CloseParen) {
            vec![]
        } else {
            self.parse_args_list()
        };

        self.expect(
            TokenType::CloseParen,
            "Closing parenthesis (\")\") expected while parsing arguments.",
        );

        args
    }

    fn parse_and_statement(&mut self) -> NodeType {
        let mut left = self.parse_additive_expr();

        let current_token = self.at();

        if current_token.value == "&&" || current_token.value == "|" {
            let operator = self.eat().value;
            let right = self.parse_additive_expr();

            left = NodeType::BinaryExpr(BinaryExpr {
                left: Box::new(left),
                right: Box::new(right),
                operator,
            })
        }

        left
    }

    fn parse_try_catch_expr(&mut self) -> NodeType {
        if self.at().value != "try" {
            return self.parse_and_statement();
        }

        self.eat();

        let body = self.parse_block_statement();

        if self.at().value != "catch" {
            println!("\"try\" statement must be followed by a \"catch\" statement.");
            process::exit(1);
        }

        self.eat();

        let alternate = self.parse_block_statement();

        NodeType::TryCatchStatement(TryCatchStatement { body, alternate })
    }

    fn parse_object_expr(&mut self) -> NodeType {
        if !matches!(self.at().tok_type, TokenType::OpenBrace) {
            return self.parse_try_catch_expr();
        }

        self.eat();

        let mut properties: Vec<NodeType> = vec![];

        while self.not_eof() && !matches!(self.at().tok_type, TokenType::CloseBrace) {
            let key = self
                .expect(
                    TokenType::Identifier,
                    "Identifier expected following \"Object\" expression.",
                )
                .value;

            if matches!(self.at().tok_type, TokenType::Comma) {
                self.eat();
                properties.push(NodeType::Property(Property { key, value: None }));
                continue;
            } else if matches!(self.at().tok_type, TokenType::CloseBrace) {
                properties.push(NodeType::Property(Property { key, value: None }));
                continue;
            }

            self.expect(
                TokenType::Colon,
                "Semicolon (\";\") expected following \"identifier\" in \"Object\" expression.",
            );
            let value = self.parse_expr();

            properties.push(NodeType::Property(Property {
                key,
                value: Some(Box::new(value)),
            }));

            if !matches!(self.at().tok_type, TokenType::CloseBrace) {
                self.expect(TokenType::Comma, "Comma (\";\") or closing brace (\"}\") expected after \"property\" declaration.");
            }
        }

        self.expect(
            TokenType::CloseBrace,
            "Closing brace (\"}\") expected at the end of \"Object\" expression.",
        );
        NodeType::ObjectLiteral(ObjectLiteral { properties })
    }

    fn parse_additive_expr(&mut self) -> NodeType {
        let mut left = self.parse_multiplicative_expr();

        while vec!["+", "-", "==", "!=", "<", ">"].contains(&mut self.at().value.as_str()) {
            let operator = self.eat().value;
            let right = self.parse_multiplicative_expr();

            left = NodeType::BinaryExpr(BinaryExpr {
                left: Box::new(left),
                right: Box::new(right),
                operator,
            })
        }

        left
    }

    fn parse_multiplicative_expr(&mut self) -> NodeType {
        let mut left = self.parse_call_member_expr();

        while vec!["/", "*", "%"].contains(&mut self.at().value.as_str()) {
            let operator = self.eat().value;
            let right = self.parse_call_member_expr();

            left = NodeType::BinaryExpr(BinaryExpr {
                left: Box::new(left),
                right: Box::new(right),
                operator,
            })
        }

        left
    }

    fn parse_call_member_expr(&mut self) -> NodeType {
        let member = self.parse_member_expr();

        if matches!(self.at().tok_type, TokenType::OpenParen) {
            return self.parse_call_expr(member);
        }

        member
    }

    fn parse_call_expr(&mut self, caller: NodeType) -> NodeType {
        let mut call_expr = NodeType::CallExpr(CallExpr {
            caller: Box::new(caller),
            args: self.parse_args(),
        });

        if matches!(self.at().tok_type, TokenType::OpenParen) {
            call_expr = self.parse_call_expr(call_expr);
        }

        call_expr
    }

    fn parse_args_list(&mut self) -> Vec<NodeType> {
        let mut args = vec![self.parse_assignment_expr()];

        while matches!(self.at().tok_type, TokenType::Comma) {
            self.eat();
            args.push(self.parse_assignment_expr());
        }

        args
    }

    fn parse_member_expr(&mut self) -> NodeType {
        let mut object = self.parse_primary_expr();

        while matches!(self.at().tok_type, TokenType::Dot)
            || matches!(self.at().tok_type, TokenType::OpenBracket)
        {
            let operator = self.eat();
            let property;
            let computed;

            if matches!(operator.tok_type, TokenType::Dot) {
                computed = false;
                property = self.parse_primary_expr();

                if !matches!(property, NodeType::Identifier(_)) {
                    println!("Dot operator (\".\") is illegal without right-hand-side (<-) being an Identifier.");
                    process::exit(1);
                }
            } else {
                computed = true;
                property = self.parse_expr();

                self.expect(TokenType::CloseBracket, "Closing bracket (\"}\") expected following \"computed value\" in \"Member\" expression.");
            }

            object = NodeType::MemberExpr(MemberExpr {
                object: Box::new(object),
                property: Box::new(property),
                computed,
            });
        }

        object
    }

    fn parse_primary_expr(&mut self) -> NodeType {
        let tk = self.at().tok_type.clone();

        match tk {
            TokenType::Identifier => NodeType::Identifier(Identifier {
                symbol: self.eat().value,
            }),
            TokenType::Number => NodeType::NumericLiteral(NumericLiteral {
                value: {
                    let val = self.eat().value;

                    val.parse().expect("Failed to convert String to f32")
                },
            }),
            TokenType::String => NodeType::StringLiteral(StringLiteral {
                value: self.eat().value,
            }),
            TokenType::OpenParen => {
                self.eat();
                let value = self.parse_expr();

                self.expect(
                    TokenType::CloseParen,
                    "Unexpected token (?) found while parsing arguments.",
                );

                value
            }
            _ => {
                println!(
                    "Unexpected token found during parsing! {:?}",
                    self.at().tok_type
                );
                process::exit(1);
            }
        }
    }
}
