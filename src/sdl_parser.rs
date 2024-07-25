
use std::fs::read_to_string;
use std::sync::Arc;
use std::collections::HashMap;

use crate::prelude::Vec3;
use crate::prelude::Color;
use crate::prelude::PI;
use crate::hittable::transform::Transform;
use crate::hittable::transform::TransformStack;
use crate::hittable::transform::TfFacade;
use crate::hittable::Sphere;
use crate::hittable::Box as GeometryBox;
use crate::hittable::collection::HittableList;
use crate::hittable::csg::Difference;
use crate::hittable::Intersection;
use crate::material::Material;
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
    Light,

    Intersection,
    Difference,

    Plus,
    Minus,
    Multiply,
    Divide,


    BlockOpen,
    BlockClose,
    VectorOpen,
    VectorClose,
    ParenOpen,
    ParenClose,
    Comma,

    Translate,
    Rotate,

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

    map.insert("<".to_string(), Symbol::VectorOpen);
    map.insert(">".to_string(), Symbol::VectorClose);
    map.insert(",".to_string(), Symbol::Comma);
    map.insert("sphere".to_string(), Symbol::Sphere);
    map.insert("box".to_string(), Symbol::Box);
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

    map.insert("translate".to_string(), Symbol::Translate);
    map.insert("rotate".to_string(), Symbol::Rotate);

    map.insert("+".to_string(), Symbol::Plus);
    map.insert("-".to_string(), Symbol::Minus);
    map.insert("*".to_string(), Symbol::Multiply);
    map.insert("/".to_string(), Symbol::Divide);
    map.insert("(".to_string(), Symbol::ParenOpen);
    map.insert(")".to_string(), Symbol::ParenClose);

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
    
    let seps = [' ', ',', '<', '>', '{', '}', '+', '-', '*', '/', '\n'];

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

    println!("Line {}, Current symbol is: {}", input.current_line(), input.current_text());
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
        println!("Expected {:?}, found {}", s, input.current_text());
        false
    }
}    

// parser functions

fn parse_root(input: &mut Input, scene: &mut SceneData) -> bool {

    nextsym(input);

    parse_statement_list(input, scene)
}


fn parse_statement_list(input: &mut Input, scene: &mut SceneData) -> bool {

    println!("Line {}, parse_statement_list called", input.current_line());

    while input.pos < input.tokens.len() {
        if !parse_statement(input, scene) {
            // something went wrong
            return false;
        }
    }

    true
}

fn parse_statement(input: &mut Input, scene: &mut SceneData) -> bool {

    println!("Line {}, parse_statement: '{}'", input.current_line(), input.current_text());

    if parse_camera(input, scene) {
    }
    else if parse_light(input, scene) {
    }
    else if parse_sphere(input, scene) {
    }
    else if parse_box(input, scene) {
    }
    else if parse_difference(input, scene) {
    }
    else if parse_intersection(input, scene) {
    }
    else if input.symbol == Symbol::Eof {
        println!("EOF, stop parsing");
        return false;
    }
    else {
        println!("Line {}, Invalid statement found: {}", input.current_line(), input.current_text());
        return false;
    }
    println!("Line {}, statement done: {}", input.current_line(), input.current_text());

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
        camera.fov_angle = parse_expression(input).unwrap();
        return true;
    }
    else {
        println!("Line {}, parse_camera_vector: expected 'location' or 'look_at', found '{}'", input.current_line(), input.current_text());
    }

    false
}

fn parse_sphere(input: &mut Input, scene: &mut SceneData) -> bool {

    println!("Line {}, parse_sphere: called, current symbol is {:?}", input.current_line(), input.current_text());

    if expect_quiet(input, Symbol::Sphere) {
        if expect(input, Symbol::BlockOpen) {
            let v = parse_vector(input).unwrap();
            expect(input, Symbol::Comma);
            let r = parse_expression(input).unwrap();   

            let material =
                if let Some(material) = parse_texture(input) {
                    material
                }
                else {
                    println!("Line {}, parse_sphere: found no texture, using default diffuse white", input.current_line());
                    Arc::new(Lambertian::new(Arc::new(Color::new(1.0, 1.0, 1.0, 1.0))))
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

    println!("Line {}, parse_box: called, current symbol is {:?}", input.current_line(), input.current_text());

    if expect_quiet(input, Symbol::Box) {
        if expect(input, Symbol::BlockOpen) {
            let v1 = parse_vector(input).unwrap();
            expect(input, Symbol::Comma);
            let v2 = parse_vector(input).unwrap();

            let mut material: Arc<dyn Material> = Arc::new(Lambertian::new(Arc::new(Color::new(1.0, 1.0, 1.0, 1.0))));
            let mut tf_stack = TransformStack::new();

            loop {
                if let Some(mat) = parse_texture(input) {
                    material = mat;
                }
                else if let Some(transform) = parse_object_modifiers(input) {
                    tf_stack.push(transform);
                }
                else {
                    break;
                }
            }

            let gbox = GeometryBox::new(v1, v2, material);
            println!("parse_box: ok -> {:?}", gbox);

            let object = TfFacade::new(gbox, tf_stack);
            scene.hittables.add(object);

            expect(input, Symbol::BlockClose);

            return true;
        }
        else {
            println!("Line {}, parse_box: expected {{, found {}", input.current_line(), input.current_text());
        }
    }

    false
}


fn parse_difference(input: &mut Input, scene: &mut SceneData) -> bool {

    println!("Line {}, parse_difference: called, current symbol is {:?}", input.current_line(), input.current_text());

    if expect_quiet(input, Symbol::Difference) {
        if expect(input, Symbol::BlockOpen) {

            let mut list = SceneData::new();

            println!("parse_difference: looking for first statement");

            // we need two objects for a difference, how to deal with cameras?
            if parse_statement(input, &mut list) {

                println!("parse_difference: parsed first statement");

                if parse_statement(input, &mut list) {
                    let mut objects = list.hittables.into_objects();

                    println!("parse_difference: parsed second statement, now checking objects");

                    if objects.len() == 2  {
                        let minus = objects.remove(1);
                        let plus = objects.remove(0);

                        let diff = Difference::new(plus, minus);

                        scene.hittables.add(diff);

                        println!("Line {}, parse_difference -> ok", input.current_line());

                        expect(input, Symbol::BlockClose);

                        return true;
                    }
                    else {
                        println!("Line {}, parse_difference: need two objects for a difference, found {}", input.current_line(), objects.len());
                    }
                }
                else {
                    println!("Line {}, parse_difference: statement expected, found {}", input.current_line(), input.current_text());
                }    
            }
            else {
                println!("Line {}, parse_difference: first statement expected, found {}", input.current_line(), input.current_text());
            }
        }
        else {
            println!("Line {}, parse_difference: expected {{, found {}", input.current_line(), input.current_text());
        }
    }

    false
}


fn parse_intersection(input: &mut Input, scene: &mut SceneData) -> bool {

    println!("Line {}, parse_intersection: called, current symbol is {:?}", input.current_line(), input.current_text());

    if expect_quiet(input, Symbol::Intersection) {
        if expect(input, Symbol::BlockOpen) {

            let mut list = SceneData::new();

            println!("parse_intersection: looking for first statement");

            // we need two objects for a difference, how to deal with cameras?
            if parse_statement(input, &mut list) {

                println!("parse_intersection: parsed first statement");

                if parse_statement(input, &mut list) {
                    let mut objects = list.hittables.into_objects();

                    println!("parse_intersection: parsed second statement, now checking objects");

                    if objects.len() == 2  {
                        let o2 = objects.remove(1);
                        let o1 = objects.remove(0);

                        let intersection = Intersection::new(o1, o2);

                        scene.hittables.add(intersection);

                        // scene.hittables.add_ref(o1);
                        // scene.hittables.add_ref(o2);

                        println!("Line {}, parse_intersection -> ok", input.current_line());

                        expect(input, Symbol::BlockClose);

                        return true;
                    }
                    else {
                        println!("Line {}, parse_intersection: need two objects for a difference, found {}", input.current_line(), objects.len());
                    }
                }
                else {
                    println!("Line {}, parse_intersection: statement expected, found {}", input.current_line(), input.current_text());
                }    
            }
            else {
                println!("Line {}, parse_intersection: first statement expected, found {}", input.current_line(), input.current_text());
            }
        }
        else {
            println!("Line {}, parse_intersection: expected {{, found {}", input.current_line(), input.current_text());
        }
    }

    false
}


fn parse_object_modifiers(input: &mut Input) -> Option<Transform> {

    if let Some(v) = parse_translate(input) {
        println!("parse_object_modifiers: translate ok");
        return Some(Transform::translate(v));
    }
    else if let Some(v) = parse_rotate(input) {
        println!("parse_object_modifiers: rotate ok {:?}", v);
        return Some(Transform::rotate_by_y_axis(v.y * PI / 180.0));
        // return Some(Transform::rotate_by_y_axis(0.0));
    }

    None
}

fn parse_texture(input: &mut Input) -> Option<Arc<dyn Material>> {

    if expect_quiet(input, Symbol::Texture) {
        if expect(input, Symbol::BlockOpen) {

            let texture =
                if let Some(texture) = parse_pigment(input) {
                    texture
                } else {
                    println!("Line {}, parse_texture: no pigment found, using default white", input.current_line());
                    Arc::new(Color::new(1.0, 1.0, 1.0, 1.0))
                };

            let mut material = parse_finish(input, texture);

            expect(input, Symbol::BlockClose);

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

            let mut material: Arc<dyn Material> = Arc::new(Lambertian::new(texture.clone()));

            if expect_quiet(input, Symbol::Reflection) {
                let v = parse_float(input).unwrap();

                let metal = Arc::new(Metal::new(texture));

                println!("Line {}, parse_finish: using mixed material, reflection={}",
                         input.current_line(), v);
                
                material = Arc::new(MixedMaterial::new(metal, material, v));
            }
            
            expect(input, Symbol::BlockClose);
            return Some(material);
        }
    }
    else if expect(input, Symbol::Surface) {
        if expect(input, Symbol::BlockOpen) {

            let material: Arc<dyn Material> =
                if expect_quiet(input, Symbol::Metallic) {

                    if expect_quiet(input, Symbol::Diffuse) {
                        println!("Line {}, parse_surface: using diffuse metal", input.current_line());
                        let v = parse_float(input).unwrap();
                        Arc::new(DiffuseMetal::new(v, texture))
                    }
                    else {
                        println!("Line {}, parse_surface: using specular metal", input.current_line());
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


fn parse_surface(input: &mut Input, texture: Arc<dyn Texture>) -> Option<Arc<dyn Material>> {

    if expect(input, Symbol::Surface) {
        if expect(input, Symbol::BlockOpen) {

            let material: Arc<dyn Material> =
                if expect_quiet(input, Symbol::Metallic) {

                    if expect_quiet(input, Symbol::Diffuse) {
                        println!("Line {}, parse_surface: using diffuse metal", input.current_line());
                        let v = parse_float(input).unwrap();
                        Arc::new(DiffuseMetal::new(v, texture))
                    }
                    else {
                        println!("Line {}, parse_surface: using specular metal", input.current_line());
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


fn parse_translate(input: &mut Input) -> Option<Vec3> {

    println!("parse_translate: called");

    if expect_quiet(input, Symbol::Translate) {
        if let Some(v) = parse_vector(input) {
            return Some(v);
        }
        else {
            println!("Line {}, parse_translate: expected vector, found '{}'", input.current_line(), input.current_text());
        }
    }

    None
}


fn parse_rotate(input: &mut Input) -> Option<Vec3> {

    println!("parse_rotate: called");

    if expect_quiet(input, Symbol::Rotate) {
        if let Some(v) = parse_vector(input) {
            return Some(v);
        }
        else {
            println!("Line {}, parse_rotate: expected vector, found '{}'", input.current_line(), input.current_text());
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
            println!("Line {}, parse_color: expected color vector, but found '{}'", input.current_line(), input.current_text());
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
        println!("Line {}, parse_vector: expected <, found {}", input.current_line(), input.current_text());
    }

    None
}


fn parse_expression(input: &mut Input) -> Option<f64> {

    println!("Line {}, parse_expression: called with {}", input.current_line(), input.current_text());

    let mut e;

    if expect(input, Symbol::Minus) {
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

        if expect(input, Symbol::Minus) {
            if let Some(value) = parse_term(input) {
                e -= value;
            }    
            else {
                println!("Line {}, parse_expression: expected term, found {}", input.current_line(), input.current_text());
                return None;
            }    
        }            
        else if expect(input, Symbol::Plus) {
            if let Some(value) = parse_term(input) {
                e += value;
            }
            else {
                println!("Line {}, parse_expression: expected term, found {}", input.current_line(), input.current_text());
                return None;
            }    
        }
        else {
            // end of expression? Diagnosis?
            break;
        }
    }

    println!("Line {}, parse_expression: -> ok, value is {}", input.current_line(), e);

    Some(e)
}


fn parse_term(input: &mut Input) -> Option<f64> {
    if let Some(mut f) = parse_factor(input) {

        loop {
            if expect(input, Symbol::Multiply) {
                if let Some(value) = parse_factor(input) {
                    f *= value;
                }    
                else {
                    println!("Line {}, parse_term: expected factor, found {}", input.current_line(), input.current_text());
                    return None;
                }    
            }            
            else if expect(input, Symbol::Divide) {
                if let Some(value) = parse_factor(input) {
                    f /= value;
                }
                else {
                    println!("Line {}, parse_term: expected factor, found {}", input.current_line(), input.current_text());
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
        println!("Line {}, parse_term: expected factor, found {}", input.current_line(), input.current_text());
    }

    None
}


fn parse_factor(input: &mut Input) -> Option<f64> {
    if expect(input, Symbol::ParenOpen) {
        let e = parse_expression(input);

        if expect(input, Symbol::ParenClose) {
            return e; 
        }
        else {
            println!("Line {}, parse_factor: expected closing parenthesis, found {}", input.current_line(), input.current_text());
        }
    }
    else {
        return parse_float(input)
    }

    println!("Line {}, parse_factor: expected float value or openening parenthesis, found {}", input.current_line(), input.current_text());

    None
}

fn parse_float(input: &mut Input) -> Option<f64> {
    let v = input.current_text().parse::<f64>();
    
    if v.is_ok() {
        let v = v.unwrap();
        println!("Line {}, parse_float: -> ok, value is {}", input.current_line(), v);
        nextsym(input);
        return Some(v);
    }
    else {
        println!("Line {}, parse_float: expected float number, found {}", input.current_line(), input.current_text());
    }

    None
}