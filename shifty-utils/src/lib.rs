mod date_utils;

pub use date_utils::*;

/// Implement `From<T>` for a type which already implement From<&T>
#[macro_export]
macro_rules! derive_from_reference {
    ($from_type:ty, $impl_type:ty) => {
        impl From<$from_type> for $impl_type {
            fn from(value: $from_type) -> Self {
                Self::from(&value)
            }
        }
    };
}
#[macro_export]
macro_rules! derive_try_from_reference {
    ($from_type:ty, $impl_type:ty, $error_type:ty) => {
        impl TryFrom<$from_type> for $impl_type {
            type Error = $error_type;

            fn try_from(value: $from_type) -> Result<Self, Self::Error> {
                Self::try_from(&value)
            }
        }
    };
}

/// A type which indicates if a resource is loaded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LazyLoad<K, T> {
    /// The resource is loaded.
    pub key: K,
    pub value: Option<T>,
}
impl<K, T> LazyLoad<K, T> {
    /// Create new object with the key.
    pub fn new(key: K) -> Self {
        Self { key, value: None }
    }

    pub fn is_loaded(&self) -> bool {
        self.value.is_some()
    }

    pub fn get(&self) -> Option<&T> {
        self.value.as_ref()
    }

    pub fn set(&mut self, value: T) {
        self.value = Some(value);
    }

    pub fn key(&self) -> &K {
        &self.key
    }
}

#[cfg(test)]
mod tests {
    struct FirstStruct(i32);
    struct SecondStruct(i32);

    impl From<&FirstStruct> for SecondStruct {
        fn from(value: &FirstStruct) -> Self {
            SecondStruct(value.0)
        }
    }
    derive_from_reference!(FirstStruct, SecondStruct);

    #[test]
    fn test_derive_from_reference() {
        let first = FirstStruct(42);
        let second: SecondStruct = first.into();
        assert_eq!(second.0, 42);
    }
}
