use std::rc::Rc;

use crate::base_types::ImStr;

#[derive(Clone)]
pub struct DropdownEntry {
    pub text: ImStr,
    pub action: Rc<dyn Fn() + 'static>,
    pub disabled: bool,
}
impl PartialEq for DropdownEntry {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text
    }
}
//impl From<(&'static str, Box<dyn Fn() + 'static>)> for DropdownEntry {
//    fn from((text, action): (&'static str, Box<dyn Fn()>)) -> Self {
//        Self {
//            text: ImStr::from(text),
//            action,
//        }
//    }
//}
impl<F> From<(&'static str, F)> for DropdownEntry
where
    F: Fn() + 'static,
{
    fn from(tuple: (&'static str, F)) -> Self {
        DropdownEntry {
            text: ImStr::from(tuple.0),
            action: Rc::new(tuple.1),
            disabled: false,
        }
    }
}
impl<F> From<(&'static str, F, bool)> for DropdownEntry
where
    F: Fn() + 'static,
{
    fn from(triple: (&'static str, F, bool)) -> Self {
        DropdownEntry {
            text: ImStr::from(triple.0),
            action: Rc::new(triple.1),
            disabled: triple.2,
        }
    }
}
impl<F> From<(ImStr, F)> for DropdownEntry
where
    F: Fn() + 'static,
{
    fn from(tuple: (ImStr, F)) -> Self {
        DropdownEntry {
            text: tuple.0,
            action: Rc::new(tuple.1),
            disabled: false,
        }
    }
}
impl<F> From<(ImStr, F, bool)> for DropdownEntry
where
    F: Fn() + 'static,
{
    fn from(triple: (ImStr, F, bool)) -> Self {
        DropdownEntry {
            text: ImStr::from(triple.0),
            action: Rc::new(triple.1),
            disabled: triple.2,
        }
    }
}

#[derive(Clone, Default, PartialEq)]
pub struct Dropdown {
    pub x: f64,
    pub y: f64,
    pub entries: Rc<[DropdownEntry]>,
}
