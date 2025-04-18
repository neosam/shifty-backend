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
