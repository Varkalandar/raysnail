use raysnail::sdl_parser::SdlParser;
use raysnail::sdl_parser::SceneData;

pub fn main() {

    let result = SdlParser::parse("sdl/example.sdl");

    if let Err(message) = result {
        println!("Could not parse scene data.");
    } 
}