use crate::Composition;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct State<'a> {
    pub composition: Option<&'a Composition>,
    pub rendered: String,
}
