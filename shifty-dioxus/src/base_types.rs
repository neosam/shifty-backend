use core::fmt;
use std::{
    fmt::{Display, Formatter},
    rc::Rc,
};

use dioxus::{dioxus_core::AttributeValue, prelude::IntoAttributeValue};

#[derive(Clone, PartialEq, Eq, Debug, Hash, PartialOrd, Ord, Default)]
pub struct ImStr {
    pub(crate) inner: Rc<str>,
}

impl ImStr {
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    pub fn as_rc(&self) -> Rc<str> {
        self.inner.clone()
    }
}

impl From<String> for ImStr {
    fn from(s: String) -> Self {
        Self {
            inner: Rc::<str>::from(s),
        }
    }
}

impl From<&str> for ImStr {
    fn from(s: &str) -> Self {
        Self {
            inner: Rc::<str>::from(s),
        }
    }
}

impl From<Rc<str>> for ImStr {
    fn from(s: Rc<str>) -> Self {
        Self { inner: s }
    }
}

impl IntoAttributeValue for ImStr {
    fn into_value(self) -> AttributeValue {
        AttributeValue::Text(self.inner.to_string())
    }
}

impl Display for ImStr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}
