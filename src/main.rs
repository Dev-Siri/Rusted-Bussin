use std::{
    env,
    error::Error,
    io::{self, Write},
    process,
};

use frontend::parser::{Parse, Parser};
use runtime::environment::create_global_env;
use utils::transcriber;

use crate::runtime::interpreter;

mod frontend;
mod runtime;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    match args.get(1) {
        Some(filename) => run(filename).await?,
        None => repl().await?,
    }

    Ok(())
}

async fn run<'a>(filename: &String) -> Result<(), Box<dyn Error>> {
    let mut parser = Parser::new();
    let env = create_global_env()?;

    let mut input = tokio::fs::read_to_string(filename).await?;

    if filename.ends_with(".bsx") {
        input = transcriber::transcribe(input);
    }

    let program = parser.create_ast(input);
    let result = interpreter::evaluate(&program, &env)?;

    println!("{:?}", result);
    Ok(())
}

async fn repl() -> Result<(), Box<dyn Error>> {
    let mut parser = Parser::new();

    println!("Repl v1.0 (Rusted Bussin)");

    let mut input = String::new();

    loop {
        let env = create_global_env()?;
        io::stdout().flush().expect("Failed to flush io::stdout");

        print!("> ");
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        if input == "" || input == "exit" {
            process::exit(1);
        }

        let program = parser.create_ast(input.clone());
        let result = interpreter::evaluate(&program, &env)?;

        println!("{:?}", result);
    }
}
