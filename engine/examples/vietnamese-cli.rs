use std::io::{self, Read};
use vietnamese_engine::{Config, Engine, Input, Result};

fn main() -> io::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let mut engine = Engine::new(Config::default());

    for character in input.chars() {
        let key = match character {
            ' ' => Input::Space,
            '\n' => Input::Enter,
            '\t' => Input::Tab,
            character => Input::Character(character),
        };
        match engine.input(key) {
            Result::Changed => eprint!("\r\x1b[2K{}", engine.rendered()),
            Result::Commit(text) => print!("{text}"),
            Result::Noop | Result::Forward => {}
        }
    }
    if let Result::Commit(text) = engine.commit() {
        print!("{text}");
    }
    Ok(())
}
