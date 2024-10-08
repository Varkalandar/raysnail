
use std::fs::read_to_string;
use std::sync::Arc;
use std::collections::HashMap;
use std::fmt::Formatter;
use std::fmt::Debug;

use crate::prelude::Vec3;
use crate::prelude::Color;
use crate::prelude::PI;

use crate::hittable::Hittable;
use crate::hittable::transform::Transform;
use crate::hittable::transform::TransformStack;
use crate::hittable::transform::TfFacade;
use crate::hittable::Sphere;
use crate::hittable::Box as GeometryBox;
use crate::hittable::geometry::Quadric;
use crate::hittable::collection::HittableList;
use crate::hittable::csg::Difference;
use crate::hittable::Intersection;

use crate::material::Material;
use crate::material::CommonMaterialSettings;
use crate::material::Lambertian;
use crate::material::Metal;
use crate::material::DiffuseMetal;
use crate::material::MixedMaterial;

use crate::texture::Checker;
use crate::texture::Texture;


// All data parsed from the scene definition
#[derive(Debug)]
pub struct SceneData {
    pub camera: Option <CameraData>,
    pub hittables: HittableList,
    pub lights: Vec<LightData>,
}

impl SceneData {
    pub fn new() -> Self {
        SceneData {
            camera: None,
            hittables: HittableList::default(),
            lights: Vec::new(),
        }        
    }
}

#[derive(Debug, PartialEq)]
pub struct CameraData {
    pub location: Vec3,
    pub look_at: Vec3,
    pub fov_angle: f64,
}

#[derive(Debug, PartialEq)]
pub struct LightData {
    pub location: Vec3, 
    pub color: Color, 
}

#[derive(Debug)]
struct Token {
    text: String,
    line: u32,
}

enum DeclaredEntity {
    Light(LightData),
    Camera(CameraData),
    Hittable(Arc<dyn Hittable>),
    Directive(String),
    Float(f64),
    Vector(Vec3),
    Invalid,
}

impl Debug for DeclaredEntity {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "DeclaredEntity",
        ))
    }
}


#[derive(Debug)]
struct Input {
    symbol_map: HashMap<String, Symbol>,
    pos: usize,
    tokens: Vec<Token>,
    
    symbol: Symbol,

    declares: HashMap <String, DeclaredEntity>,
    loops: Vec<usize>,  // to mark input positions of the start of loop statements
}

impl Input {

    fn current_line(&self) -> u32 {
        if self.pos < self.tokens.len() {        
            self.tokens[self.pos].line
        }
        else {
            self.tokens.len() as u32
        }
    }

    fn current_text(&self) -> &String {
        if self.pos < self.tokens.len() {        
            &self.tokens[self.pos].text
        }
        else {
            &self.tokens[self.tokens.len() - 1].text
        }
    }
}


#[derive(Debug, PartialEq, Clone)]
enum Symbol {
    Camera,
    Location,
    LookAt,
    Sphere,
    Box,
    Quadric,
    Light,

    Intersection,
    Difference,
    Object,

    Plus,
    Minus,
    Multiply,
    Divide,
    Equal,

    BlockOpen,
    BlockClose,
    VectorOpen,
    VectorClose,
    ParenOpen,
    ParenClose,
    Comma,
    Semicolon,

    Translate,
    Rotate,
    Scale,

    Texture,
    Pigment,
    Finish,
    Surface,

    Metallic,
    Reflection,
    Color,
    Rgb,
    Angle,
    Diffuse,
    Phong,
    PhongSize,

    Checker,
    
    Declare,
    While,
    End,
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
            declares: HashMap::new(),
            loops: Vec::new(),
        };

        let mut scene = SceneData::new();

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
    
    map.insert("intersection".to_string(), Symbol::Intersection);
    map.insert("difference".to_string(), Symbol::Difference);
    map.insert("object".to_string(), Symbol::Object);

    map.insert("<".to_string(), Symbol::VectorOpen);
    map.insert(">".to_string(), Symbol::VectorClose);
    map.insert(",".to_string(), Symbol::Comma);
    map.insert(";".to_string(), Symbol::Semicolon);
    map.insert("sphere".to_string(), Symbol::Sphere);
    map.insert("box".to_string(), Symbol::Box);
    map.insert("quadric".to_string(), Symbol::Quadric);
    map.insert("light".to_string(), Symbol::Light);

    map.insert("texture".to_string(), Symbol::Texture);
    map.insert("pigment".to_string(), Symbol::Pigment);
    map.insert("finish".to_string(), Symbol::Finish);
    map.insert("surface".to_string(), Symbol::Surface);

    map.insert("reflection".to_string(), Symbol::Reflection);
    map.insert("metallic".to_string(), Symbol::Metallic);
    map.insert("color".to_string(), Symbol::Color);
    map.insert("rgb".to_string(), Symbol::Rgb);
    map.insert("checker".to_string(), Symbol::Checker);
    map.insert("angle".to_string(), Symbol::Angle);
    map.insert("diffuse".to_string(), Symbol::Diffuse);
    map.insert("phong".to_string(), Symbol::Phong);
    map.insert("phong_size".to_string(), Symbol::PhongSize);

    map.insert("translate".to_string(), Symbol::Translate);
    map.insert("rotate".to_string(), Symbol::Rotate);
    map.insert("scale".to_string(), Symbol::Scale);

    map.insert("+".to_string(), Symbol::Plus);
    map.insert("-".to_string(), Symbol::Minus);
    map.insert("*".to_string(), Symbol::Multiply);
    map.insert("/".to_string(), Symbol::Divide);
    map.insert("(".to_string(), Symbol::ParenOpen);
    map.insert(")".to_string(), Symbol::ParenClose);
    map.insert("=".to_string(), Symbol::Equal);

    map.insert("#declare".to_string(), Symbol::Declare);
    map.insert("#while".to_string(), Symbol::While);
    map.insert("#end".to_string(), Symbol::End);

    map
}


fn push_non_empty(v: &mut Vec<Token>, value: &str, line: u32) {
    let t = value.trim();
    if t.len() > 0 {
        v.push(Token {text: t.to_string(), line});
    }
}


fn strip_line_comments(line: &String) -> String {

    let mut parts = line.split("//");
    if let Some(part) = parts.next() {
        return part.to_string();
    }

    return line.to_string();
}

fn tokenize(line_in: &String, line_no: u32) -> Vec<Token> {

    let line = strip_line_comments(line_in);
    
    let seps = [' ', ',', ';', '(', ')', '<', '>', '{', '}', '+', '-', '*', '/', '\n'];

    let mut v = Vec::new();

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
    }
    
    v
}


fn read_tokens(filename: &str) -> Vec<Token> {
    let mut result = Vec::new();
    let mut line_no = 1; // Editors start with line 1 

    // generate start token to fill (unused) position 0
    result.push(Token {text: "START".to_string(), line: 0});

    for line in read_to_string(filename).unwrap().lines() {
        let token_line = &line.to_string();
        let tokens = tokenize(token_line, line_no);
        for token in tokens {
            // println!("Token = '{}' len = {}", token, token.len());
            result.push(token);
        }

        line_no += 1;
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
        input.symbol = to_symbol(&input.symbol_map, &token.text);
    }
    else {
        input.symbol = Symbol::Eof;
    }

    //marked println!("Line {}, Current symbol is: {}", input.current_line(), input.current_text());
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
        //marked println!("Expected {:?}, found {}", s, input.current_text());
        false
    }
}    

// parser functions

fn parse_root(input: &mut Input, scene: &mut SceneData) -> bool {

    nextsym(input);

    parse_statement_list(input, scene)
}


fn parse_statement_list(input: &mut Input, scene: &mut SceneData) -> bool {

    //marked println!("Line {}, parse_statement_list called", input.current_line());

    while input.pos < input.tokens.len() {
        let entity = parse_statement(input);

        match entity {
            DeclaredEntity::Hittable(object) => {
                scene.hittables.add_ref(object);
            },
            DeclaredEntity::Light(light) => {
                scene.lights.push(light);
            },
            DeclaredEntity::Camera(camera) => {
                scene.camera = Some(camera);
            },
            DeclaredEntity::Directive(_ident) => {
                // nothing to do here
            },
            DeclaredEntity::Float(_v) => {
                // nothing to do here
            },
            DeclaredEntity::Vector(_v) => {
                // nothing to do here
            },
            DeclaredEntity::Invalid => {
                // something went wrong
                return false;
            }
        }
    }

    true
}

fn parse_statement(input: &mut Input) -> DeclaredEntity {

    //marked println!("Line {}, parse_statement: '{}'", input.current_line(), input.current_text());

    let entity = parse_camera(input);
    match entity { DeclaredEntity::Invalid => {}, _ => { return entity; },}

    let entity = parse_light(input);
    match entity { DeclaredEntity::Invalid => {}, _ => { return entity; },}

    let entity = parse_sphere(input);
    match entity { DeclaredEntity::Invalid => {}, _ => { return entity; },}

    let entity = parse_box(input);
    match entity { DeclaredEntity::Invalid => {}, _ => { return entity; },}

    let entity = parse_quadric(input);
    match entity { DeclaredEntity::Invalid => {}, _ => { return entity; },}

    let entity = parse_object(input);
    match entity { DeclaredEntity::Invalid => {}, _ => { return entity; },}

    let entity = parse_difference(input);
    match entity { DeclaredEntity::Invalid => {}, _ => { return entity; },}

    let entity = parse_intersection(input);
    match entity { DeclaredEntity::Invalid => {}, _ => { return entity; },}

    let entity = parse_declare(input);
    match entity { DeclaredEntity::Invalid => {}, _ => { return entity; },}

    let entity = parse_while(input);
    match entity { DeclaredEntity::Invalid => {}, _ => { return entity; },}

    let entity = parse_end(input);
    match entity { DeclaredEntity::Invalid => {}, _ => { return entity; },}

    if input.symbol == Symbol::Eof {
        //marked println!("EOF, stop parsing");
        return DeclaredEntity::Invalid;
    }

    //marked println!("Line {}, Invalid statement found: {}", input.current_line(), input.current_text());
    return DeclaredEntity::Invalid;
}


fn parse_camera(input: &mut Input) -> DeclaredEntity {
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
                    //marked println!("Line {}, parse_camera: expected camera vector or }}, found {}", input.current_line(), input.current_text());
                    return DeclaredEntity::Invalid;
                }
            }

            //marked println!("parse_camera: ok -> {:?}", camera);
            nextsym(input);

            return DeclaredEntity::Camera(camera);
        }
        //marked println!("Line {}, parse_camera: expected {{, found {}", input.current_line(), input.current_text());
    }

    // println!("Line {}, parse_camera: statement is no camera, {}", input.current_line(), input.current_text());

    DeclaredEntity::Invalid
}


fn parse_light(input: &mut Input) -> DeclaredEntity {
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
                    //marked println!("Line {}, parse_light: expected color vector, found {}", input.current_line(), input.current_text());
                    return DeclaredEntity::Invalid;
                }
            }
            else {
                //marked println!("Line {}, parse_light: expected location vector, found {}", input.current_line(), input.current_text());
                return DeclaredEntity::Invalid;
            }

            //marked println!("parse_light: ok -> {:?}", light);

            return DeclaredEntity::Light(light);
        }
        //marked println!("Line {}, parse_camera: expected {{, found {}", input.current_line(), input.current_text());
    }

    DeclaredEntity::Invalid
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
        camera.fov_angle = parse_expression(input).unwrap();
        return true;
    }
    else {
        //marked println!("Line {}, parse_camera_vector: expected 'location' or 'look_at', found '{}'", input.current_line(), input.current_text());
    }

    false
}

fn parse_sphere(input: &mut Input) -> DeclaredEntity {

    //marked println!("Line {}, parse_sphere: called, current symbol is {:?}", input.current_line(), input.current_text());

    if expect_quiet(input, Symbol::Sphere) {
        if expect(input, Symbol::BlockOpen) {
            let v = parse_vector(input).unwrap();
            expect(input, Symbol::Comma);
            let r = parse_expression(input).unwrap();   

            let material = parse_texture(input);
            let stack = parse_object_modifiers(input);

            let sphere = Arc::new(Sphere::new(v, r, material));

            //marked println!("parse_sphere: ok -> {:?}", sphere);

            expect(input, Symbol::BlockClose);

            return DeclaredEntity::Hittable(build_transform_facade(stack, sphere));
        }
        else {
            //marked println!("Line {}, parse_sphere: expected {{, found {}", input.current_line(), input.current_text());
        }
    }
    else {
        //marked println!("Line {}, parse_sphere: not a sphere", input.current_line());
    }

    DeclaredEntity::Invalid
}


fn parse_box(input: &mut Input) -> DeclaredEntity {

    //marked println!("Line {}, parse_box: called, current symbol is {:?}", input.current_line(), input.current_text());

    if expect_quiet(input, Symbol::Box) {
        if expect(input, Symbol::BlockOpen) {
            let v1 = parse_vector(input).unwrap();
            expect(input, Symbol::Comma);
            let v2 = parse_vector(input).unwrap();

            let material = parse_texture(input);
            let stack = parse_object_modifiers(input);

            let gbox = Arc::new(GeometryBox::new(v1, v2, material));
            //marked println!("parse_box: ok -> {:?}", gbox);

            expect(input, Symbol::BlockClose);

            return DeclaredEntity::Hittable(build_transform_facade(stack, gbox));
        }
        else {
            //marked println!("Line {}, parse_box: expected {{, found {}", input.current_line(), input.current_text());
        }
    }

    DeclaredEntity::Invalid
}


fn parse_quadric(input: &mut Input) -> DeclaredEntity {

    //marked println!("Line {}, parse_quadric: called, current symbol is {:?}", input.current_line(), input.current_text());

    if expect_quiet(input, Symbol::Quadric) {
        if expect(input, Symbol::BlockOpen) {
            let v1 = parse_vector(input).unwrap();
            expect(input, Symbol::Comma);
            let v2 = parse_vector(input).unwrap();
            expect(input, Symbol::Comma);
            let v3 = parse_vector(input).unwrap();
            expect(input, Symbol::Comma);
            let j = parse_expression(input).unwrap();

            let material = parse_texture(input);
            let stack = parse_object_modifiers(input);

            let quadric = 
                Quadric::new(v1.x, v2.x, v2.y, v3.x, v1.y, v2.z, v3.y, v1.z, v3.z, j,
                             material);

            //marked println!("parse_quadric: ok -> {:?}", quadric);

            expect(input, Symbol::BlockClose);

            return DeclaredEntity::Hittable(build_transform_facade(stack, Arc::new(quadric)));
        }
        else {
            //marked println!("Line {}, parse_quadric: expected {{, found {}", input.current_line(), input.current_text());
        }
    }

    DeclaredEntity::Invalid
}


fn parse_object(input: &mut Input) -> DeclaredEntity {

    //marked println!("Line {}, parse_object: called, current symbol is {:?}", input.current_line(), input.current_text());

    if expect_quiet(input, Symbol::Object) {
        if expect(input, Symbol::BlockOpen) {

            let ident_opt = parse_identifier(input);

            if ident_opt.is_some() {
                let ident = ident_opt.unwrap();
                //marked println!("parse_object: identifier is {:?}, now looking for declared data", ident);

                let stack = parse_object_modifiers(input);

                expect(input, Symbol::BlockClose);

                let entity = input.declares.get(&ident);

                match entity.unwrap() {
                    DeclaredEntity::Hittable(object) => {
                        //marked println!("parse_object: got valid entity");
                        //marked println!("Line {}, parse_object -> ok", input.current_line());

                        let copy = object.clone();
                        return DeclaredEntity::Hittable(build_transform_facade(stack, copy));
                    },
                    _ => {
                        //marked println!("Line {}, parse_object: got no entity for identifier", input.current_line());
                    }
                }
            }
            else {
                //marked println!("Line {}, parse_object: undeclared identifier {:?}", input.current_line(), ident_opt);
            }
        }
        else {
            //marked println!("Line {}, parse_object: expected {{, found {}", input.current_line(), input.current_text());
        }
    }

    DeclaredEntity::Invalid
}


fn parse_difference(input: &mut Input) -> DeclaredEntity {

    //marked println!("Line {}, parse_difference: called, current symbol is {:?}", input.current_line(), input.current_text());

    if expect_quiet(input, Symbol::Difference) {
        if expect(input, Symbol::BlockOpen) {

            //marked println!("parse_difference: looking for first statement");

            // we need two objects for a difference, how to deal with cameras?
            if let DeclaredEntity::Hittable(plus) = parse_statement(input) {

                //marked println!("parse_difference: parsed first statement");

                if let DeclaredEntity::Hittable(minus) = parse_statement(input) {

                    //marked println!("parse_difference: parsed second statement, now checking objects");

                    let material = parse_texture(input);
                    let stack = parse_object_modifiers(input);

                    let difference = Arc::new(Difference::new(plus, minus, material));

                    //marked println!("Line {}, parse_difference -> ok", input.current_line());

                    expect(input, Symbol::BlockClose);

                    return DeclaredEntity::Hittable(build_transform_facade(stack, difference));
                }
                else {
                    //marked println!("Line {}, parse_difference: second statement expected, found {}", input.current_line(), input.current_text());
                }    
            }
            else {
                //marked println!("Line {}, parse_difference: first statement expected, found {}", input.current_line(), input.current_text());
            }
        }
        else {
            //marked println!("Line {}, parse_difference: expected {{, found {}", input.current_line(), input.current_text());
        }
    }

    DeclaredEntity::Invalid
}


fn parse_intersection(input: &mut Input) -> DeclaredEntity {

    //marked println!("Line {}, parse_intersection: called, current symbol is {:?}", input.current_line(), input.current_text());

    if expect_quiet(input, Symbol::Intersection) {
        if expect(input, Symbol::BlockOpen) {

            //marked println!("parse_intersection: looking for first statement");

            // we need two objects for a difference, how to deal with cameras?
            if let DeclaredEntity::Hittable(o1) = parse_statement(input) {

                //marked println!("parse_intersection: parsed first statement");

                if let DeclaredEntity::Hittable(o2) = parse_statement(input) {

                    //marked println!("parse_intersection: parsed second statement, now checking objects");
                    let material = parse_texture(input);
                    let stack = parse_object_modifiers(input);

                    let intersection = Arc::new(Intersection::new(o1, o2, material));

                    //marked println!("Line {}, parse_intersection -> ok", input.current_line());

                    expect(input, Symbol::BlockClose);

                    return DeclaredEntity::Hittable(build_transform_facade(stack, intersection));
                }
                else {
                    //marked println!("Line {}, parse_intersection: second statement expected, found {}", input.current_line(), input.current_text());
                }    
            }
            else {
                //marked println!("Line {}, parse_intersection: first statement expected, found {}", input.current_line(), input.current_text());
            }
        }
        else {
            //marked println!("Line {}, parse_intersection: expected {{, found {}", input.current_line(), input.current_text());
        }
    }

    DeclaredEntity::Invalid
}


fn parse_declare(input: &mut Input) -> DeclaredEntity {

    //marked println!("Line {}, parse_declare: called, current symbol is {:?}", input.current_line(), input.current_text());

    if expect_quiet(input, Symbol::Declare) {
        if let Some(ident) = parse_identifier(input) {
            if expect(input, Symbol::Equal) {

                //marked println!("Line {}, parse_declare: checking value {:?}", input.current_line(), input.current_text());

                // test non-statement cases first

                if let Some(v) = parse_expression(input) {
                    nextsym(input);
                    expect(input, Symbol::Semicolon);
                    //marked println!("Line {}, parse_declare -> scalar expression ok {:?}, current symbol is {:?}", input.current_line(), v, input.current_text());
                    input.declares.insert(ident.to_string(), DeclaredEntity::Float(v));
                    return DeclaredEntity::Directive("#declare".to_string());
                }
                else if let Some(v) = parse_vector(input) {
                    expect(input, Symbol::Semicolon);
                    //marked println!("Line {}, parse_declare -> vector expression ok {:?}, current symbol is {:?}", input.current_line(), v, input.current_text());
                    input.declares.insert(ident.to_string(), DeclaredEntity::Vector(v));
                    return DeclaredEntity::Directive("#declare".to_string());
                }
                else {
                    let entity = parse_statement(input);
                    input.declares.insert(ident.to_string(), entity);
                    //marked println!("Line {}, parse_declare -> statement ok, current symbol is {:?}", input.current_line(), input.current_text());
                    return DeclaredEntity::Directive("#declare".to_string());
                }
            }
        }
    }

    //marked println!("Line {}, parse_declare: failed, current symbol is {:?}", input.current_line(), input.current_text());
    DeclaredEntity::Invalid
}


fn parse_while(input: &mut Input) -> DeclaredEntity {

    //marked println!("Line {}, parse_while: called, current symbol is {:?}", input.current_line(), input.current_text());

    let loop_start = input.pos;

    if expect_quiet(input, Symbol::While) {
        if expect(input, Symbol::ParenOpen) {

            //marked println!("Line {}, parse_while: checking condition {:?}", input.current_line(), input.current_text());

            if let Some(v1) = parse_expression(input) {
                nextsym(input);

                expect(input, Symbol::VectorOpen);

                if let Some(v2) = parse_expression(input) {
                    nextsym(input);

                    if v1 < v2 {
                        //marked println!("Line {}, parse_while -> comparision ok {:?} < {:?}, current symbol is {:?}", input.current_line(), v1, v2, input.current_text());

                        input.loops.push(loop_start);
                    }
                    else {
                        //marked println!("Line {}, parse_while -> goto loop end", input.current_line());

                        fast_forward_to_end(input);
                    }

                    return DeclaredEntity::Directive("#while".to_string());                    
                }
            }
        }
    }

    //marked println!("Line {}, parse_while: failed, current symbol is {:?}", input.current_line(), input.current_text());
    DeclaredEntity::Invalid
}


fn fast_forward_to_end(input: &mut Input) {
    while input.symbol != Symbol::End {
        nextsym(input);
    }

    nextsym(input);
}


fn parse_end(input: &mut Input) -> DeclaredEntity {
    if expect_quiet(input, Symbol::End) {
        let loop_start = input.loops.pop().unwrap();

        // Continue parsing at loop start
        input.pos = loop_start - 1;
        nextsym(input);

        //marked println!("Line {}, parse_end: ok, loop start is {}, current symbol is {:?}", input.pos, input.current_line(), input.current_text());
        return DeclaredEntity::Directive("#end".to_string());
    }

    //marked println!("Line {}, parse_end: failed, current symbol is {:?}", input.current_line(), input.current_text());
    DeclaredEntity::Invalid
}


fn build_transform_facade(stack: TransformStack, hittable: Arc<dyn Hittable>) ->  Arc<dyn Hittable> {

    if stack.len() > 0 {
        return Arc::new(TfFacade::new(hittable, stack))
    }

    hittable
}


fn parse_object_modifiers(input: &mut Input) -> TransformStack {

    let mut stack = TransformStack::new();

    loop {
        if let Some(v) = parse_translate(input) {
            //marked println!("parse_object_modifiers: translate ok");
            stack.push(Transform::translate(v));
        }
        else if let Some(v) = parse_rotate(input) {
            //marked println!("parse_object_modifiers: rotate ok {:?}", v);

            if v.x != 0.0 {
                stack.push(Transform::rotate_by_x_axis(v.x * PI / 180.0));
            }

            if v.y != 0.0 {
                stack.push(Transform::rotate_by_y_axis(v.y * PI / 180.0));
            }

            if v.z != 0.0 {
                stack.push(Transform::rotate_by_z_axis(v.z * PI / 180.0));
            }
        }
        else if let Some(v) = parse_scale(input) {
            //marked println!("parse_object_modifiers: scale ok {:?}", v);
            stack.push(Transform::scale(v));
        }
        else {
            break;
        }
    }

    stack
}

fn parse_texture(input: &mut Input) -> Option<Arc<dyn Material>> {

    if expect_quiet(input, Symbol::Texture) {
        if expect(input, Symbol::BlockOpen) {

            let texture =
                if let Some(texture) = parse_pigment(input) {
                    texture
                } else {
                    //marked println!("Line {}, parse_texture: no pigment found, using default white", input.current_line());
                    Arc::new(Color::new(1.0, 1.0, 1.0, 1.0))
                };

            let material = parse_finish(input, texture);

            expect(input, Symbol::BlockClose);

            //marked println!("Line {}, parse_texture -> ok", input.current_line());

            return material;
        }
    }

    None
}

fn parse_pigment(input: &mut Input) -> Option<Arc<dyn Texture>> {

    if expect(input, Symbol::Pigment) {
        if expect(input, Symbol::BlockOpen) {
            if let Some(color) = parse_color(input) {
                expect_quiet(input, Symbol::Rgb);   // should this be made mandatory?

                expect(input, Symbol::BlockClose);
                return Some(Arc::new(color));
            }
            else if let Some(colors) = parse_checker(input) {
                expect(input, Symbol::BlockClose);
                return Some(Arc::new(Checker::new(colors.0, colors.1, 2.0)));
            }
        }
    }

    None
}


fn parse_finish(input: &mut Input, texture: Arc<dyn Texture>) -> Option<Arc<dyn Material>> {

    if expect(input, Symbol::Finish) {
        if expect(input, Symbol::BlockOpen) {

            let mut phong = 0.0;
            let mut phong_size = 40.0;
            let mut reflection = 0.0;

            loop {
                if expect_quiet(input, Symbol::Reflection) {
                    reflection = parse_float(input).unwrap();
                }
                else if expect_quiet(input, Symbol::Phong) {
                    phong = parse_float(input).unwrap();
                }
                else if expect_quiet(input, Symbol::PhongSize) {
                    phong_size = parse_float(input).unwrap();
                }
                else {
                    break;
                }
            }
            expect(input, Symbol::BlockClose);

            let material: Arc<dyn Material> =
                if reflection == 0.0 {
                    let mut lambertian = Lambertian::new(texture);
                    lambertian.set(settings(phong, phong_size));

                    Arc::new(lambertian)
                }
                else {
                    let mut lambertian = Lambertian::new(texture.clone());
                    lambertian.set(settings(phong, phong_size));

                    let mut metal = Metal::new(texture);
                    metal.set(settings(phong, phong_size));

                    //marked println!("Line {}, parse_finish: using mixed material, reflection={}", input.current_line(), reflection);

                    Arc::new(MixedMaterial::new(Arc::new(metal), Arc::new(lambertian), reflection))
                };

            return Some(material);
        }
    }
    else if expect(input, Symbol::Surface) {
        if expect(input, Symbol::BlockOpen) {

            let material: Arc<dyn Material> =
                if expect_quiet(input, Symbol::Metallic) {

                    if expect_quiet(input, Symbol::Diffuse) {
                        //marked println!("Line {}, parse_surface: using diffuse metal", input.current_line());
                        let v = parse_float(input).unwrap();
                        Arc::new(DiffuseMetal::new(v, texture))
                    }
                    else {
                        //marked println!("Line {}, parse_surface: using specular metal", input.current_line());
                        Arc::new(Metal::new(texture))
                    }
                }
                else {
                    Arc::new(Lambertian::new(texture))
                };
            
            expect(input, Symbol::BlockClose);
            return Some(material);
        }
    }

    // Lambertian is default
    Some(Arc::new(Lambertian::new(texture)))
}


fn settings(phong_factor: f64, phong_exponent: f64) -> CommonMaterialSettings {
    let mut settings = CommonMaterialSettings::new();
    
    if phong_factor > 0.0 {
        settings.phong_factor = phong_factor * 4.0;
        settings.phong_exponent = (phong_exponent * 0.1) as i32;
    }

    settings
}

fn parse_surface(input: &mut Input, texture: Arc<dyn Texture>) -> Option<Arc<dyn Material>> {

    if expect(input, Symbol::Surface) {
        if expect(input, Symbol::BlockOpen) {

            let material: Arc<dyn Material> =
                if expect_quiet(input, Symbol::Metallic) {

                    if expect_quiet(input, Symbol::Diffuse) {
                        //marked println!("Line {}, parse_surface: using diffuse metal", input.current_line());
                        let v = parse_float(input).unwrap();
                        Arc::new(DiffuseMetal::new(v, texture))
                    }
                    else {
                        //marked println!("Line {}, parse_surface: using specular metal", input.current_line());
                        Arc::new(Metal::new(texture))
                    }
                }
                else {
                    Arc::new(Lambertian::new(texture))
                };
            
            expect(input, Symbol::BlockClose);
            return Some(material);
        }

        // Lambertian is default
        return Some(Arc::new(Lambertian::new(texture)));
    }
    None
}


fn parse_checker(input: &mut Input) -> Option<(Color, Color)> {
    if expect_quiet(input, Symbol::Checker) {
        if let Some(color1) = parse_color(input) {

            expect(input, Symbol::Comma);

            if let Some(color2) = parse_color(input) {
                //marked println!("parse_checker: -> ok ({:?}, {:?})", color1, color2);
                return Some((color1, color2));
            }
            else {
                //marked println!("Line {}, parse_checker: expected color, found '{}'", input.current_line(), input.current_text());
            }
        }
        else {
            //marked println!("Line {}, parse_checker: expected color, found '{}'", input.current_line(), input.current_text());
        }
    }
    None
}


fn parse_identifier(input: &mut Input) -> Option<String> {
    let ident = input.current_text().to_string();
    nextsym(input);

    //marked println!("Line {}, parse_identifier: found '{}'", input.current_line(), ident);

    Some(ident)
}


fn parse_translate(input: &mut Input) -> Option<Vec3> {

    //marked println!("parse_translate: called");

    if expect_quiet(input, Symbol::Translate) {
        if let Some(v) = parse_vector(input) {
            return Some(v);
        }
        else {
            //marked println!("Line {}, parse_translate: expected vector, found '{}'", input.current_line(), input.current_text());
        }
    }

    None
}


fn parse_rotate(input: &mut Input) -> Option<Vec3> {

    //marked println!("parse_rotate: called");

    if expect_quiet(input, Symbol::Rotate) {
        if let Some(v) = parse_vector(input) {
            return Some(v);
        }
        else {
            //marked println!("Line {}, parse_rotate: expected vector, found '{}'", input.current_line(), input.current_text());
        }
    }

    None
}


fn parse_scale(input: &mut Input) -> Option<Vec3> {

    //marked println!("parse_scale: called");

    if expect_quiet(input, Symbol::Scale) {
        if let Some(v) = parse_vector(input) {
            return Some(v);
        }
        else if let Some(v) = parse_float(input) {
            return Some(Vec3::new(v, v, v));
        }
        else {
            //marked println!("Line {}, parse_scale: expected float or vector, found '{}'", input.current_line(), input.current_text());
        }
    }

    None
}


fn parse_color(input: &mut Input) -> Option<Color> {
    if expect_quiet(input, Symbol::Color) {
        expect_quiet(input, Symbol::Rgb);   // should this be made mandatory?

        if let Some(v) = parse_vector(input) {
            return Some(Color::new64(v.x, v.y, v.z, 1.0))
        } else {
            //marked println!("Line {}, parse_color: expected color vector, but found '{}'", input.current_line(), input.current_text());
        }
    }
    None
}

fn parse_vector(input: &mut Input) -> Option<Vec3> {
    if expect(input, Symbol::VectorOpen) {

        let v1 = parse_expression(input).unwrap();
        expect(input, Symbol::Comma);

        let v2 = parse_expression(input).unwrap();
        expect(input, Symbol::Comma);
        
        let v3 = parse_expression(input).unwrap();
        expect(input, Symbol::VectorClose);

        return Some(Vec3::new(v1, v2, v3));
    }
    else {
        //marked println!("Line {}, parse_vector: expected <, found {}", input.current_line(), input.current_text());
    }

    None
}


fn parse_expression(input: &mut Input) -> Option<f64> {

    //marked println!("Line {}, parse_expression called, current symbol is {}", input.current_line(), input.current_text());

    let mut e;

    if expect_quiet(input, Symbol::Minus) {
        if let Some(value) = parse_term(input) {
            e = -value;
        }
        else {
            return None;
        }
    }
    else {
        if let Some(value) = parse_term(input) {
            e = value;
        }
        else {
            return None;
        }
    }

    loop {

        if expect_quiet(input, Symbol::Minus) {
            if let Some(value) = parse_term(input) {
                e -= value;
            }    
            else {
                //marked println!("Line {}, parse_expression: expected term, found {}", input.current_line(), input.current_text());
                return None;
            }    
        }            
        else if expect_quiet(input, Symbol::Plus) {
            if let Some(value) = parse_term(input) {
                e += value;
            }
            else {
                //marked println!("Line {}, parse_expression: expected term, found {}", input.current_line(), input.current_text());
                return None;
            }    
        }
        else {
            // end of expression? Diagnosis?
            break;
        }
    }

    //marked println!("Line {}, parse_expression: -> ok, value is {}", input.current_line(), e);

    Some(e)
}


fn parse_term(input: &mut Input) -> Option<f64> {
    if let Some(mut f) = parse_factor(input) {

        loop {
            if expect_quiet(input, Symbol::Multiply) {
                if let Some(value) = parse_factor(input) {
                    f *= value;
                }    
                else {
                    //marked println!("Line {}, parse_term: expected factor, found {}", input.current_line(), input.current_text());
                    return None;
                }    
            }            
            else if expect_quiet(input, Symbol::Divide) {
                if let Some(value) = parse_factor(input) {
                    f /= value;
                }
                else {
                    //marked println!("Line {}, parse_term: expected factor, found {}", input.current_line(), input.current_text());
                    return None;
                }    
            }
            else {
                // end of expression? Diagnosis?
                break;
            }            
        }

        return Some(f);
    }
    else {
//        println!("Line {}, parse_term: expected factor, found {}", input.current_line(), input.current_text());
    }

    None
}


fn parse_factor(input: &mut Input) -> Option<f64> {
    if expect_quiet(input, Symbol::ParenOpen) {
        let e = parse_expression(input);

        if expect(input, Symbol::ParenClose) {
            return e; 
        }
        else {
            //marked println!("Line {}, parse_factor: expected closing parenthesis, found {}", input.current_line(), input.current_text());
        }
    }
    else {
        let ident = input.current_text();
        //marked println!("Line {}, parse_factor: testing identifier: {}", input.current_line(), ident);

        let symbol_opt = input.declares.get(ident);

        if symbol_opt.is_some() {
            let e = symbol_opt.unwrap(); 
            match e {
                DeclaredEntity::Float(f) => {
                    let v = *f;
                    nextsym(input);
                    return Some(v);
                }
                _ => {
                    //marked println!("Line {}, parse_factor: expected declared float, found {:?}", input.current_line(), e);
                }
            }
        }
        
        return parse_float(input)
    }

    //marked println!("Line {}, parse_factor: expected float value or openening parenthesis, found {}", input.current_line(), input.current_text());

    None
}

fn parse_float(input: &mut Input) -> Option<f64> {
    let v = input.current_text().parse::<f64>();
    
    if v.is_ok() {
        let v = v.unwrap();
        //marked println!("Line {}, parse_float: -> ok, value is {}", input.current_line(), v);
        nextsym(input);
        return Some(v);
    }
    else {
//        println!("Line {}, parse_float: expected float number, found {}", input.current_line(), input.current_text());
    }

    None
}