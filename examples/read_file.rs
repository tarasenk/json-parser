use json_parser::{Value, from_str};
use std::env;
use std::fs;
use std::process;

fn main() {
    let file_path = env::args()
        .nth(1)
        .unwrap_or_else(|| "data.json".to_string());

    let raw_json = fs::read_to_string(&file_path).unwrap_or_else(|err| {
        eprintln!("error reading '{}': {}", file_path, err);
        process::exit(1);
    });

    match from_str(&raw_json) {
        Ok(parsed) => {
            println!("{:#?}", parsed);

            if let Value::Object(ref map) = parsed {
                if let Some(Value::String(name)) = map.get("name") {
                    println!("name: {}", name);
                }
            }
        }
        Err(err) => {
            eprintln!("parse error: {:?}", err);
            process::exit(1);
        }
    }
}
