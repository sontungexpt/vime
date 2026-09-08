#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Input {
    Character(char),
    Backspace,
    Delete,
    Left,
    Right,
    Enter,
    Escape,
    Tab,
    Space,
}
