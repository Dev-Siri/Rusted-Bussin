use std::{
    error::Error,
    io,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use regex::Regex;

use crate::runtime::values::{mk_null, mk_number, mk_string, ValueType};

pub fn print_values(args: Vec<ValueType>) -> Result<ValueType, Box<dyn Error>> {
    for arg in args {
        println!("{}", match_type(arg));
    }

    Ok(mk_null())
}

pub fn match_type(arg: ValueType) -> String {
    match arg {
        ValueType::StringVal(string_val) => string_val.value,
        ValueType::NumberVal(number_val) => number_val.value.to_string(),
        ValueType::BooleanVal(boolean_val) => boolean_val.value.to_string(),
        ValueType::NullVal => "null".to_string(),
        ValueType::ObjectVal(object_val) => {
            let mut obj = String::new();

            obj += "{\n";

            for (key, value) in object_val.properties.iter() {
                obj += format!("  {}: {},\n", key, match_type(value.clone())).as_str();
            }

            obj += "}";

            obj
        }
        ValueType::FunctionVal(function_val) => format!(
            "function {{\n  name: {},\n  body: {:?},\n  internal: false\n}}",
            function_val.name, function_val.body
        ),
        ValueType::NativeFnVal(native_fn_val) => {
            format!("function {} {{ [native code] }}", native_fn_val.name)
        }
    }
}

pub fn exec(args: Vec<ValueType>) -> Result<ValueType, Box<dyn Error>> {
    let cmd = match args.get(0).expect("Command message is required") {
        ValueType::StringVal(string_val) => string_val.value.clone(),
        _ => Err("Command message must be of type StringVal")?,
    };

    let output = String::from_utf8(
        Command::new(cmd)
            .output()
            .expect("Failed to run system command")
            .stdout,
    )
    .expect("Failed to convert command output from Vec<u8> to String");

    Ok(mk_string(output))
}

pub fn input(args: Vec<ValueType>) -> Result<ValueType, Box<dyn Error>> {
    let cmd = match args.get(0).expect("Command message is required") {
        ValueType::StringVal(string_val) => string_val.value.clone(),
        _ => Err("Command message must be of type StringVal")?,
    };

    let mut input = String::new();

    print!("{}", cmd);
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to get input");

    Ok(if input == "" {
        mk_null()
    } else {
        mk_string(input)
    })
}

pub fn math_sqrt(args: Vec<ValueType>) -> Result<ValueType, Box<dyn Error>> {
    let num = match args.get(0).expect("Number required for math.sqrt()") {
        ValueType::NumberVal(number_val) => number_val.value.clone(),
        _ => Err("Num must be of type NumberVal")?,
    };

    Ok(mk_number(Some(f32::sqrt(num))))
}

pub fn math_random(args: Vec<ValueType>) -> Result<ValueType, Box<dyn Error>> {
    let num1 = match args
        .get(0)
        .expect("First number is required for math.random()")
    {
        ValueType::NumberVal(number_val) => number_val.value.clone(),
        _ => Err("Num must be of type NumberVal")?,
    };
    let num2 = match args
        .get(1)
        .expect("Second number is required for math.random()")
    {
        ValueType::NumberVal(number_val) => number_val.value.clone(),
        _ => Err("Num must be of type NumberVal")?,
    };

    let min = f32::ceil(num1);
    let max = f32::floor(num2);

    let random = f32::floor(rand::random::<f32>() * (max - min + 1.0) + min);

    Ok(mk_number(Some(random)))
}

pub fn math_round(args: Vec<ValueType>) -> Result<ValueType, Box<dyn Error>> {
    let num = match args.get(0).expect("Number required for math.round()") {
        ValueType::NumberVal(number_val) => number_val.value.clone(),
        _ => Err("Num must be of type NumberVal")?,
    };

    Ok(mk_number(Some(f32::round(num))))
}

pub fn math_ceil(args: Vec<ValueType>) -> Result<ValueType, Box<dyn Error>> {
    let num = match args.get(0).expect("Number required for math.ceil()") {
        ValueType::NumberVal(number_val) => number_val.value.clone(),
        _ => Err("Num must be of type NumberVal")?,
    };

    Ok(mk_number(Some(f32::ceil(num))))
}

pub fn math_abs(args: Vec<ValueType>) -> Result<ValueType, Box<dyn Error>> {
    let num = match args.get(0).expect("Number required for math.abs()") {
        ValueType::NumberVal(number_val) => number_val.value.clone(),
        _ => Err("Num must be of type NumberVal")?,
    };

    Ok(mk_number(Some(f32::abs(num))))
}

pub fn strcon(args: Vec<ValueType>) -> Result<ValueType, Box<dyn Error>> {
    let mut res = String::new();

    for arg in args {
        let arg_str = match_type(arg);

        res += arg_str.as_str();
    }

    Ok(mk_string(res))
}

pub fn format(args: Vec<ValueType>) -> Result<ValueType, Box<dyn Error>> {
    let mut_args = &mut args.clone();
    let str = match_type(mut_args.remove(0));

    let mut res: String = String::new();

    for arg in args {
        let arg_str = match_type(arg);

        let re = Regex::new(r"\$\{\}").expect("Invalid regex pattern");
        res = re.replace(str.as_str(), arg_str).to_string();
    }

    if matches!(mut_args.get(0), None) {
        Err("2nd parameter in format! missing.")?
    }

    Ok(mk_string(res))
}

pub fn time_function(_: Vec<ValueType>) -> Result<ValueType, Box<dyn Error>> {
    let current_time = SystemTime::now();

    // Calculate the number of seconds since the Unix epoch
    let epoch_time = current_time
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    Ok(mk_number(Some(epoch_time as f32)))
}
