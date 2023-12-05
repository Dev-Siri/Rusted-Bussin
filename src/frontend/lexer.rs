use std::{collections::HashMap, process};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Literal Types
    Number,
    Identifier,
    String,
    // Keywords
    Let,
    Const,
    Fn,
    If,
    Else,
    For,

    // Grouping * Operators
    BinaryOperator,
    Equals,           // =
    Comma,            // ,
    Colon,            // :
    Semicolon,        // ;
    Dot,              // .
    OpenParen,        // (
    CloseParen,       // )
    OpenBrace,        // {
    CloseBrace,       // }
    OpenBracket,      // [
    CloseBracket,     // ]
    Greater,          // >
    Lesser,           // <
    EqualsCompare,    // ==
    NotEqualsCompare, // !=
    Exclamation,      // !
    And,              // &&
    Ampersand,        // &
    Bar,              // |
    EOF,              // Signified the end of file.
}

const KEYWORDS: &[(&str, TokenType); 6] = &[
    ("let", TokenType::Let),
    ("const", TokenType::Const),
    ("fn", TokenType::Fn),
    ("if", TokenType::If),
    ("else", TokenType::Else),
    ("for", TokenType::For),
];

const TOKEN_CHARS: &[(char, TokenType); 18] = &[
    ('(', TokenType::OpenParen),
    (')', TokenType::CloseParen),
    ('{', TokenType::OpenBrace),
    ('}', TokenType::CloseBrace),
    ('[', TokenType::OpenBracket),
    (']', TokenType::CloseBracket),
    ('+', TokenType::BinaryOperator),
    ('-', TokenType::BinaryOperator),
    ('*', TokenType::BinaryOperator),
    ('%', TokenType::BinaryOperator),
    ('/', TokenType::BinaryOperator),
    ('<', TokenType::Lesser),
    ('>', TokenType::Greater),
    ('.', TokenType::Dot),
    (';', TokenType::Semicolon),
    (':', TokenType::Colon),
    (',', TokenType::Comma),
    ('|', TokenType::Bar),
];

#[derive(Debug)]
pub struct Token {
    pub value: String,
    pub tok_type: TokenType,
}

fn token(value: Option<&str>, tok_type: TokenType) -> Token {
    Token {
        value: value.map_or_else(|| String::new(), String::from),
        tok_type,
    }
}

fn is_alpha(src: &str, is_first_char: bool) -> bool {
    if is_first_char {
        return src
            .chars()
            .next()
            .map_or(false, |c| c.is_ascii_alphabetic() || c == '_');
    }

    src.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn is_skippable(str: String) -> bool {
    str == " " || str == "\n" || str == "\t" || str == "\r"
}

fn is_int(str: &String) -> bool {
    str.chars().all(|c| c.is_digit(10))
}

fn str_to_first_char(str: &String) -> char {
    str.chars()
        .next()
        .expect("Failed to convert &str to first char")
}

fn first_char_as_str(str: &Vec<String>) -> String {
    str.get(0).expect("Failed to get char at pos 0").clone()
}

pub fn tokenize(source_code: String) -> Vec<Token> {
    let token_chars_map: HashMap<char, TokenType> = TOKEN_CHARS.iter().cloned().collect();
    let keywords_map: HashMap<&str, TokenType> = KEYWORDS.iter().cloned().collect();

    let mut tokens: Vec<Token> = vec![];
    let mut src: Vec<String> = source_code
        .split("")
        .flat_map(|s| s.chars().map(|c| c.to_string()))
        .collect();

    while src.len() > 0 {
        let src_char_as_str = first_char_as_str(&src);
        let c = str_to_first_char(&src_char_as_str);

        // can you understand this? me neither, but it works.
        if is_int(&c.to_string())
            || (c == '-' && is_int(src.get(1).expect("Failed to get char at pos 1")))
        {
            let mut num = src.remove(0);
            let mut period = false;

            'num_loop: while src.len() > 0
                && (is_int(&str_to_first_char(&first_char_as_str(&src)).to_string())
                    || str_to_first_char(&first_char_as_str(&src)) == '.')
            {
                if str_to_first_char(&first_char_as_str(&src)) == '.' && !period {
                    period = true;
                    num += src.remove(0).as_str();
                } else if is_int(&str_to_first_char(&first_char_as_str(&src)).to_string()) {
                    num += src.remove(0).as_str();
                } else {
                    break 'num_loop;
                }
            }

            tokens.push(token(Some(num.as_str()), TokenType::Number));
        } else if let Some(token_type) = token_chars_map.get(&c) {
            tokens.push(token(Some(src.remove(0).as_str()), token_type.clone()));
        } else {
            match c {
                '=' => {
                    src.remove(0);

                    if first_char_as_str(&src) == "=" {
                        src.remove(0);
                        tokens.push(token(Some("=="), TokenType::EqualsCompare));
                    } else {
                        tokens.push(token(Some("="), TokenType::Equals));
                    }
                }
                '&' => {
                    src.remove(0);

                    if first_char_as_str(&src) == "&" {
                        src.remove(0);
                        tokens.push(token(Some("&&"), TokenType::And));
                    } else {
                        tokens.push(token(Some("&"), TokenType::Ampersand));
                    }
                }
                '!' => {
                    src.remove(0);

                    if first_char_as_str(&src) == "=" {
                        src.remove(0);
                        tokens.push(token(Some("!="), TokenType::NotEqualsCompare));
                    } else {
                        tokens.push(token(Some("!"), TokenType::Exclamation));
                    }
                }
                '"' => {
                    let mut str = String::new();
                    src.remove(0);

                    while src.len() > 0 && first_char_as_str(&src) != "\"" {
                        str += src.remove(0).as_str();
                    }

                    src.remove(0);
                    tokens.push(token(Some(str.as_str()), TokenType::String));
                }
                _ => {
                    if is_alpha(c.to_string().as_str(), true) {
                        let mut ident = String::new();
                        ident += src.remove(0).as_str();

                        while src.len() > 0 && is_alpha(&first_char_as_str(&src), false) {
                            ident += src.remove(0).as_str();
                        }

                        if let Some(reserved) = keywords_map.get(&ident.as_str()) {
                            tokens.push(token(Some(ident.as_str()), reserved.clone()));
                        } else {
                            tokens.push(token(Some(ident.as_str()), TokenType::Identifier));
                        }
                    } else if is_skippable(first_char_as_str(&src)) {
                        src.remove(0);
                    } else {
                        println!(
                            "Unrecognizable character found in sources: {}",
                            first_char_as_str(&src)
                        );

                        process::exit(1);
                    }
                }
            }
        }
    }

    tokens.push(token(Some("EndOfFile"), TokenType::EOF));

    tokens
}
