
use std::fs::read_to_string;
use std::str::FromStr;


#[derive(Debug)]
struct Input {
    pos: usize,
    tokens: Vec<String>,
    
    symbol: Symbol,
    symbol_text: String,
}

#[derive(Debug, PartialEq)]
enum Symbol {
    Camera,
    BlockOpen,
    BlockClose,
    VectorOpen,
    VectorClose,
    None
}

#[derive(Debug)]
pub struct SdlParser {

}

impl SdlParser {



    pub fn parse(filename: &str) {
        let mut input = Input {
            pos: 0,
            tokens: read_tokens(filename),
            symbol: Symbol::None,
            symbol_text: "".to_string(), 
        };

        parse_root(&mut input);
    }
}


fn push_non_empty(v: &mut Vec<String>, value: &str) {
    let t = value.trim();
    if t.len() > 0 {
        v.push(t.to_string());
    }
}


fn tokenize(line: &String) -> Vec<String> {
    
    let seps = [' ', ',', '<', '>', '{', '}', '\n'];

    let mut v = Vec::new();
    
    for part in line.split_inclusive(&seps[..]) {
        if part.ends_with(seps) {
            
            let mut chars = part.chars();
            let sep = chars.next_back();
            
            push_non_empty(&mut v, &sep.unwrap().to_string());

            let left = part.strip_suffix(seps);

            push_non_empty(&mut v, left.unwrap());
        }
        else {
            if part.len() > 0 {
                push_non_empty(&mut v, part);
            }
        }
    }
    
    v
}


fn read_tokens(filename: &str) -> Vec<String> {
    let mut result = Vec::new();

    for line in read_to_string(filename).unwrap().lines() {
        let token_line = &line.to_string();
        let tokens = tokenize(token_line);
        for token in tokens {
            println!("Token = '{}' len = {}", token, token.len());
            result.push(token);
        }
    }

    result
}




// lexer functions

fn to_symbol(token: &String) -> Symbol {
    if token == "camera" {
        Symbol::Camera
    }
    else if token == "{" {
        Symbol::BlockOpen
    }
    else if token == "}" {
        Symbol::BlockClose
    }
    else if token == "<" {
        Symbol::VectorOpen
    }
    else if token == ">" {
        Symbol::VectorClose
    }
    else {
        Symbol::None
    }
}

fn nextsym(input: &mut Input) {

    let token = &input.tokens[input.pos];

    input.symbol = to_symbol(token);
    input.symbol_text = token.to_string();

    input.pos += 1;
}

fn accept(input: &mut Input, s: Symbol) -> bool {
    if input.symbol == s {
        nextsym(input);
        true
    }
    else {
        false
    }
}

fn expect(input: &mut Input, s: Symbol) -> bool {
    if accept(input, s) {
        true
    }
    else {
        false
    }
}    


// parser functions

fn parse_root(input: &mut Input) {
    nextsym(input);

    parse_statement(input);
}

fn parse_statement(input: &mut Input) -> bool {
    if parse_camera(input) {

        return true;
    }

    println!("No valid statement found: {}", input.symbol_text);
    false
}


fn parse_camera(input: &mut Input) -> bool {
    if expect(input, Symbol::Camera) {
        if expect(input, Symbol::BlockOpen) {
            parse_vector(input);

            return true;
        }
    }

    false
}

fn parse_vector(input: &mut Input) {
    if expect(input, Symbol::VectorOpen) {
        let value_opt = parse_float(input);
    }
    else {
        println!("parse_camera: expected <, got {}", input.symbol_text);
    }
}


fn parse_float(input: &mut Input) -> Result <f64, <f64 as FromStr>::Err> {
    input.symbol_text.parse::<f64>()
}