
use std::fs::read_to_string;
use std::str::FromStr;
use std::sync::Arc;
use std::collections::HashMap;

use crate::prelude::Vec3;
use crate::prelude::Color;
use crate::hittable::Sphere;
use crate::hittable::Box as GeometryBox;
use crate::hittable::collection::HittableList;
use crate::material::Material;
use crate::material::Lambertian;
use crate::texture::Checker;
use crate::texture::Texture;


// All data parsed from the scene definition
#[derive(Debug)]
pub struct SceneData {
    pub camera: Option <CameraData>,
    pub hittables: HittableList,
    pub lights: Vec<LightData>,
}


#[derive(Debug)]
pub struct CameraData {
    pub location: Vec3,
    pub look_at: Vec3,
    pub fov_angle: f64,
}

#[derive(Debug)]
pub struct LightData {
    pub location: Vec3, 
    pub color: Color, 
}

#[derive(Debug)]
struct Token {
    text: String,
    line: u32,
}

#[derive(Debug)]
struct Input {
    symbol_map: HashMap<String, Symbol>,
    pos: usize,
    tokens: Vec<Token>,
    
    symbol: Symbol,
}

impl Input {
    fn current_line(&self) -> u32 {
        self.tokens[pos].line
    }

    fn current_text(&self) -> &String {
        &self.tokens[pos].text
    }
}


#[derive(Debug, PartialEq, Clone)]
enum Symbol {
    Camera,
    Location,
    LookAt,
    Sphere,
    Box,
    Light,
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
    Angle,

    Checker,
    
    Id,
    Eof,
    None
}

#[derive(Debug)]
pub struct SdlParser {
}

impl SdlParser {

    pub fn parse(filename: &str) -> Result<SceneData, String> {
        let mut input = Input {
            symbol_map: build_symbol_map(),
            pos: 0,
            tokens: read_tokens(filename),
            symbol: Symbol::None,
            symbol_text: "".to_string(), 
        };

        let mut scene = SceneData {
            camera: None,
            hittables: HittableList::default(),
            lights: Vec::new(),
        };

        if !parse_root(&mut input, &mut scene) {
            return Err("Parse error".to_string());
        }

        return Ok(scene);
    }
}

fn build_symbol_map() -> HashMap<String, Symbol> {
    let mut map = HashMap::new();

    map.insert("camera".to_string(), Symbol::Camera);
    map.insert("look_at".to_string(), Symbol::LookAt);
    map.insert("location".to_string(), Symbol::Location);
    map.insert("{".to_string(), Symbol::BlockOpen);
    map.insert("}".to_string(), Symbol::BlockClose);
    
    map.insert("<".to_string(), Symbol::VectorOpen);
    map.insert(">".to_string(), Symbol::VectorClose);
    map.insert(",".to_string(), Symbol::Comma);
    map.insert("sphere".to_string(), Symbol::Sphere);
    map.insert("box".to_string(), Symbol::Box);
    map.insert("light".to_string(), Symbol::Light);

    map.insert("texture".to_string(), Symbol::Texture);
    map.insert("pigment".to_string(), Symbol::Pigment);
    map.insert("finish".to_string(), Symbol::Finish);
    map.insert("color".to_string(), Symbol::Color);
    map.insert("rgb".to_string(), Symbol::Rgb);
    map.insert("checker".to_string(), Symbol::Checker);
    map.insert("angle".to_string(), Symbol::Angle);

    map
}


fn push_non_empty(v: &mut Vec<Token>, value: &str, line: u32) {
    let t = value.trim();
    if t.len() > 0 {
        v.push(t.to_string());
    }
}


fn tokenize(line: &String) -> Vec<Token> {
    
    let seps = [' ', ',', '<', '>', '{', '}', '\n'];

    let mut v = Vec::new();
    let mut line_no = 0;

    for part in line.split_inclusive(&seps[..]) {
        if part.ends_with(seps) {
            
            let mut chars = part.chars();
            let sep = chars.next_back();
            let left = part.strip_suffix(seps);

            push_non_empty(&mut v, left.unwrap(), line_no);
            push_non_empty(&mut v, &sep.unwrap().to_string(), line_no);
        }
        else {
            if part.len() > 0 {
                push_non_empty(&mut v, part, line_no);
            }
        }

        line_no += 1;
    }
    
    v
}


fn read_tokens(filename: &str) -> Vec<Token> {
    let mut result = Vec::new();

    // generate start token to fill (unused) position 0
    result.push(Token {text: "START".to_string(), line: 0});

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

fn to_symbol(map: &HashMap<String, Symbol>, token: &String) -> Symbol {

    let option = map.get(token);

    if let Some(symbol) = option {
        return symbol.clone()
    }

    Symbol::Id
}

fn nextsym(input: &mut Input) {
    input.pos += 1;

    if input.pos < input.tokens.len() {
        let token = &input.tokens[input.pos];
        input.symbol = to_symbol(&input.symbol_map, token);

    }
    else {
        input.symbol = Symbol::Eof;
        input.symbol_token = Token {text: "EOF".to_string(), line: 0 };
    }

    println!("Current symbol is: {}", input.current_line(), input.current_text());
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
        println!("Expected {:?}, found {}", s, input.current_line(), input.current_text());
        false
    }
}    

// parser functions

fn parse_root(input: &mut Input, scene: &mut SceneData) -> bool {

    nextsym(input);

    parse_statement(input, scene)
}

fn parse_statement(input: &mut Input, scene: &mut SceneData) -> bool {
    while input.pos < input.tokens.len() {

        println!("Line {}, parse_statement: {:?} ('{}') ", input.symbol, input.current_line(), input.current_text());


        if parse_camera(input, scene) {
        }
        else if parse_light(input, scene) {
        }
        else if parse_sphere(input, scene) {
        }
        else if parse_box(input, scene) {
        }
        else if input.symbol == Symbol::Eof {
            println!("EOF, stop parsing");
            break;
        }
        else {
            println!("Invalid statement found: {}", input.current_line(), input.current_text());
            return false;
        }
    }

    true
}


fn parse_camera(input: &mut Input, scene: &mut SceneData) -> bool {
    if expect_quiet(input, Symbol::Camera) {

        // println!("Line {}, parse_camera: parsing camera data");

        if expect(input, Symbol::BlockOpen) {
            
            let mut camera = CameraData {
                location: Vec3::default(), 
                look_at: Vec3::default(), 
                fov_angle: 60.0,
            };

            while input.symbol != Symbol::BlockClose {
                let ok = parse_camera_item(input, &mut camera);

                if !ok {
                    println!("Line {}, parse_camera: expected camera vector or }}, found {}", input.current_line(), input.current_text());
                    return false;
                }
            }

            println!("parse_camera: ok -> {:?}", camera);
            scene.camera = Some(camera);
            nextsym(input);

            return true;
        }
        println!("Line {}, parse_camera: expected {{, found {}", input.current_line(), input.current_text());
    }

    // println!("Line {}, parse_camera: statement is no camera, {}", input.current_line(), input.current_text());

    false
}


fn parse_light(input: &mut Input, scene: &mut SceneData) -> bool {
    if expect_quiet(input, Symbol::Light) {

        if expect(input, Symbol::BlockOpen) {
            
            let mut light = LightData {
                location: Vec3::default(), 
                color: Color::default(), 
            };

            if let Some(location) = parse_vector(input) {
                expect(input, Symbol::Comma);
                if let Some(color) = parse_color(input) {

                    expect(input, Symbol::BlockClose);
                    light.location = location;
                    light.color = color;
                }
                else {
                    println!("Line {}, parse_light: expected color vector, found {}", input.current_line(), input.current_text());
                    return false;
                }
            }
            else {
                println!("Line {}, parse_light: expected location vector, found {}", input.current_line(), input.current_text());
                return false;
            }

            println!("parse_light: ok -> {:?}", light);
            scene.lights.push(light);

            return true;
        }
        println!("Line {}, parse_camera: expected {{, found {}", input.current_line(), input.current_text());
    }

    false
}



fn parse_camera_item(input: &mut Input, camera: &mut CameraData) -> bool {

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
    else if input.symbol == Symbol::Angle {
        nextsym(input);
        camera.fov_angle = parse_float(input).unwrap();
        return true;
    }
    else {
        println!("Line {}, parse_camera_vector: expected 'location' or 'look_at', found '{}'", input.current_line(), input.current_text());
    }

    false
}

fn parse_sphere(input: &mut Input, scene: &mut SceneData) -> bool {

    println!("Line {}, parse_sphere: called, current symbol is {:?}", input.symbol);

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
                    println!("Line {}, parse_sphere: found no texture, using default diffuse white");
                    Arc::new(Lambertian::new(Box::new(Color::new(1.0, 1.0, 1.0))))
                };

            let sphere = Sphere::new(v, r, material);

            println!("parse_sphere: ok -> {:?}", sphere);
            scene.hittables.add(sphere);

            expect(input, Symbol::BlockClose);

            return true;
        }
        else {
            println!("Line {}, parse_sphere: expected {{, found {}", input.current_line(), input.current_text());
        }
    }

    false
}

fn parse_box(input: &mut Input, scene: &mut SceneData) -> bool {

    println!("Line {}, parse_box: called, current symbol is {:?}", input.symbol);

    if expect_quiet(input, Symbol::Box) {
        if expect(input, Symbol::BlockOpen) {
            let v1 = parse_vector(input).unwrap();
            expect(input, Symbol::Comma);
            let v2 = parse_vector(input).unwrap();

            let material =
                if let Some(material) = parse_object_modifiers(input) {
                    material
                }
                else {
                    println!("Line {}, parse_box: found no texture, using default diffuse white");
                    Arc::new(Lambertian::new(Box::new(Color::new(1.0, 1.0, 1.0))))
                };

            let gbox = GeometryBox::new(v1, v2, material);

            println!("parse_box: ok -> {:?}", gbox);
            scene.hittables.add(gbox);

            expect(input, Symbol::BlockClose);

            return true;
        }
        else {
            println!("Line {}, parse_box: expected {{, found {}", input.current_line(), input.current_text());
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
                    println!("Line {}, parse_texture: no pigment found, using default white");
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
            if let Some(color) = parse_color(input) {
                expect_quiet(input, Symbol::Rgb);   // should this be made mandatory?

                expect(input, Symbol::BlockClose);
                return Some(Box::new(color));
            }
            else if let Some(colors) = parse_checker(input) {
                expect(input, Symbol::BlockClose);
                return Some(Box::new(Checker::new(colors.0, colors.1, 2.0)));
            }
        }
    }

    None
}

fn parse_checker(input: &mut Input) -> Option<(Color, Color)> {
    if expect_quiet(input, Symbol::Checker) {
        if let Some(color1) = parse_color(input) {

            expect(input, Symbol::Comma);

            if let Some(color2) = parse_color(input) {
                println!("parse_checker: -> ok ({:?}, {:?})", color1, color2);
                return Some((color1, color2));
            }
            else {
                println!("Line {}, parse_checker: expected color, found '{}'", input.current_line(), input.current_text());
            }
        }
        else {
            println!("Line {}, parse_checker: expected color, found '{}'", input.current_line(), input.current_text());
        }
    }
    None
}

fn parse_color(input: &mut Input) -> Option<Color> {
    if expect_quiet(input, Symbol::Color) {
        expect_quiet(input, Symbol::Rgb);   // should this be made mandatory?

        if let Some(v) = parse_vector(input) {
            return Some(Color::new(v.x, v.y, v.z))
        } else {
            println!("Line {}, parse_color: expected color vector, but found '{}'", input.current_line(), input.current_text());
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
        println!("Line {}, parse_vector: expected <, found {}", input.current_line(), input.current_text());
    }

    None
}


fn parse_float(input: &mut Input) -> Result <f64, <f64 as FromStr>::Err> {
    let v = input.current_text().parse::<f64>();
    
    if v.is_err() {
        println!("Line {}, parse_float: expected float number, found {}", input.current_line(), input.current_text());
    }

    nextsym(input);

    v
}