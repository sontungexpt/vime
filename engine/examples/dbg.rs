use vietnamese_engine::{Engine, Input};
fn main() {
    let mut engine = Engine::default();
    for ch in "nguowif".chars() {
        engine.input(Input::Character(ch));
    }
    println!(
        "raw={} rendered={}",
        engine.keystrokes().to_string(),
        engine.rendered()
    );
    // replicate telex oo case too
    let mut e2 = Engine::default();
    for ch in "oo".chars() {
        e2.input(Input::Character(ch));
    }
    println!(
        "oo raw={} rendered={}",
        e2.keystrokes().to_string(),
        e2.rendered()
    );
}
