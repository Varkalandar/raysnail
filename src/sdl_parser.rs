
use std::fs::read_to_string;
use std::str::FromStr;
use std::sync::Arc;

use crate::prelude::Vec3;
use crate::hittable::Sphere;
use crate::prelude::Color;
use crate::material::Lambertian;
use crate::hittable::collection::HittableList;
use crate::texture::Texture;
use crate::material::Material;


// All data parsed from the scene definition
#[derive(Debug)]
pub struct SceneData {
    pub camera: Option <CameraData>,
    pub hittables: HittableList,
}


#[derive(Debug)]
pub struct CameraData {
    pub location: Vec3,
    pub look_at: Vec3,
}


#[derive(Debug)]
struct Input {
    pos: usize,
    tokens: Vec<String>,
    
    symbol: Symbol,
    symbol_text: String,
}

#[derive(Debug, PartialEq, Clone)]
enum Symbol {
    Camera,
    Location,
    LookAt,
    Sphere,
    BlockOpen,
    BlockClose,
    VectorOpen,
    VectorClose,
    Comma,

    Texture,
    Pigment,
    Finish,
    Color,
    Rgb,

    Eof,
    None
}

#[derive(Debug)]
pub struct SdlParser {

}

impl SdlParser {

    pub fn parse(filename: &str) -> SceneData {
        let mut input = Input {
            pos: 0,
            tokens: read_tokens(filename),
            symbol: Symbol::None,
            symbol_text: "".to_string(), 
        };

        let mut scene = SceneData {
            camera: None,
            hittables: HittableList::default(),
        };

        parse_root(&mut input, &mut scene);

        return scene;
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
            let left = part.strip_suffix(seps);

            push_non_empty(&mut v, left.unwrap());
            push_non_empty(&mut v, &sep.unwrap().to_string());
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
            // println!("Token = '{}' len = {}", token, token.len());
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
    else if token == "look_at" {
        Symbol::LookAt
    }
    else if token == "location" {
        Symbol::Location
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
    else if token == "," {
        Symbol::Comma
    }
    else if token == "sphere" {
        Symbol::Sphere
    }
    else if token == "texture" {
        Symbol::Texture
    }
    else if token == "pigment" {
        Symbol::Pigment
    }
    else if token == "finish" {
        Symbol::Finish
    }
    else if token == "color" {
        Symbol::Color
    }
    else if token == "rgb" {
        Symbol::Rgb
    }
    else {
        // println!("Unknown token: {}", token);
        Symbol::None
    }
}

fn nextsym(input: &mut Input) {
    if input.pos < input.tokens.len() {
        let token = &input.tokens[input.pos];

        input.symbol = to_symbol(token);
        input.symbol_text = token.to_string();

        input.pos += 1;
    }
    else {
        input.symbol = Symbol::Eof;
        input.symbol_text = "EOF".to_string();
    }

    println!("Current symbol is: {}", input.symbol_text);
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

fn expect_quiet(input: &mut Input, s: Symbol) -> bool {
    if accept(input, s) {
        true
    }
    else {
        false
    }
}    

fn expect(input: &mut Input, s: Symbol) -> bool {
    if accept(input, s.clone()) {
        true
    }
    else {
        println!("Expected {:?}, found {}", s, input.symbol_text);
        false
    }
}    

// parser functions

fn parse_root(input: &mut Input, scene: &mut SceneData) {
    nextsym(input);

    parse_statement(input, scene);
}

fn parse_statement(input: &mut Input, scene: &mut SceneData) -> bool {
    while input.pos < input.tokens.len() {

        println!("parse_statement: {:?} ('{}') ", input.symbol, input.symbol_text);


        if parse_camera(input, scene) {
        }
        else if parse_sphere(input, scene) {
        }
        else if input.symbol == Symbol::Eof {
            println!("EOF, stop parsing");
            break;
        }
        else {
            println!("Invalid statement found: {}", input.symbol_text);
            return false;
        }
    }

    true
}


fn parse_camera(input: &mut Input, scene: &mut SceneData) -> bool {
    if expect_quiet(input, Symbol::Camera) {

        // println!("parse_camera: parsing camera data");

        if expect(input, Symbol::BlockOpen) {
            
            let mut camera = CameraData {location: Vec3::default(), look_at: Vec3::default(), };

            while input.symbol != Symbol::BlockClose {
                let ok = parse_camera_vector(input, &mut camera);

                if !ok {
                    println!("parse_camera: expected camera vector or }}, found {}", input.symbol_text);
                    return false;
                }
            }

            println!("parse_camera: ok -> {:?}", camera);
            scene.camera = Some(camera);
            nextsym(input);

            return true;
        }
        println!("parse_camera: expected {{, found {}", input.symbol_text);
    }

    // println!("parse_camera: statement is no camera, {}", input.symbol_text);

    false
}

fn parse_camera_vector(input: &mut Input, camera: &mut CameraData) -> bool {

    if input.symbol == Symbol::Location {
        nextsym(input);
        camera.location = parse_vector(input).unwrap();
        return true;
    }
    else if input.symbol == Symbol::LookAt {
        nextsym(input);
        camera.look_at = parse_vector(input).unwrap();
        return true;
    }
    else {
        println!("parse_camera_vector: expected 'location' or 'look_at', found '{}'", input.symbol_text);
    }

    false
}

fn parse_sphere(input: &mut Input, scene: &mut SceneData) -> bool {

    println!("parse_sphere: called, current symbol is {:?}", input.symbol);

    if expect_quiet(input, Symbol::Sphere) {
        if expect(input, Symbol::BlockOpen) {
            let v = parse_vector(input).unwrap();
            expect(input, Symbol::Comma);
            let r = parse_float(input).unwrap();   


            let material =
                if let Some(material) = parse_object_modifiers(input) {
                    material
                }
                else {
                    println!("parse_sphere: found no texture, using default diffuse white");
                    Arc::new(Lambertian::new(Box::new(Color::new(1.0, 1.0, 1.0))))
                };

            let mut sphere = Sphere::new(v, r, material);

            println!("parse_sphere: ok -> {:?}", sphere);
            scene.hittables.add(sphere);

            expect(input, Symbol::BlockClose);

            return true;
        }
        else {
            println!("parse_sphere: expected {{, found {}", input.symbol_text);
        }
    }

    false
}

fn parse_object_modifiers(input: &mut Input) -> Option<Arc<dyn Material>> {

    parse_texture(input)
}

fn parse_texture(input: &mut Input) -> Option<Arc<dyn Material>> {

    if expect_quiet(input, Symbol::Texture) {
        if expect(input, Symbol::BlockOpen) {

            let texture =
                if let Some(texture) = parse_pigment(input) {
                    texture
                } else {
                    println!("parse_texture: no pigment found, using default white");
                    Box::new(Color::new(1.0, 1.0, 1.0))
                };

            expect(input, Symbol::BlockClose);

            let material = Lambertian::new(texture);

            return Some(Arc::new(material));
        }
    }

    None
}

fn parse_pigment(input: &mut Input) -> Option<Box<dyn Texture>> {

    if expect(input, Symbol::Pigment) {
        if expect(input, Symbol::BlockOpen) {
            if expect(input, Symbol::Color) {
                expect_quiet(input, Symbol::Rgb);   // should this be made mandatory?

                if let Some(v) = parse_vector(input) {
                    expect(input, Symbol::BlockClose);
                    return Some(Box::new(Color::new(v.x, v.y, v.z)));
                } else {
                    println!("parse_pigment: expected color vector, but found '{}'", input.symbol_text);
                    return None;
                }
            }
        }
    }

    None
}



fn parse_vector(input: &mut Input) -> Option<Vec3> {
    if expect(input, Symbol::VectorOpen) {

        let v1 = parse_float(input).unwrap();
        expect(input, Symbol::Comma);

        let v2 = parse_float(input).unwrap();
        expect(input, Symbol::Comma);
        
        let v3 = parse_float(input).unwrap();
        expect(input, Symbol::VectorClose);

        return Some(Vec3::new(v1, v2, v3));
    }
    else {
        println!("parse_vector: expected <, found {}", input.symbol_text);
    }

    None
}


fn parse_float(input: &mut Input) -> Result <f64, <f64 as FromStr>::Err> {
    let v = input.symbol_text.parse::<f64>();
    
    if v.is_err() {
        println!("parse_float: expected float number, found {}", input.symbol_text);       
    }

    nextsym(input);

    v
}