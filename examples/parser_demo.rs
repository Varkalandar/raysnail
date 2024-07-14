use raysnail::sdl_parser::SdlParser;


pub fn main() -> Result<(), String> {

    SdlParser::parse("sdl/example.sdl");

    Ok(())
}