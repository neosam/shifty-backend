use std::rc::Rc;

#[derive(Clone, PartialEq)]
pub struct Tooltip {
    pub x: f64,
    pub y: f64,
    pub content: Rc<str>,
}
