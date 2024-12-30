#[macro_export]
macro_rules! gen_service_impl {
    // -----------------------------------------------------
    // 1) A helper sub-rule to generate the trait. We will
    //    reuse this logic in both the "base" pattern and
    //    the "extended" pattern with custom fields.
    // -----------------------------------------------------
    (@generate_trait $dependencies:ident, $($field_name:ident : $field_type:path),*) => {
        pub trait $dependencies {
            // You can keep or adjust these type bounds as needed
            type Context: Send + Sync + Clone + Eq + std::fmt::Debug + 'static;
            type Transaction: dao::Transaction + Send + Sync + Clone + std::fmt::Debug + 'static;

            $(
                type $field_name: $field_type + Sync + Send + 'static;
            )*
        }
    };

    // -----------------------------------------------------
    // 2) Base pattern (NO custom fields).
    //    Will match:
    //
    //    gen_service_impl! {
    //        struct ServiceName: some::Trait = MyDeps {
    //            Foo: foo::Foo = foo_field,
    //            Bar: bar::Bar = bar_field
    //        }
    //    }
    //
    // -----------------------------------------------------
    (
        struct $service_name:ident : $trait:path = $dependencies:ident {
            $(
                $field_name:ident : $field_type:path = $field_attr:ident
            ),* $(,)?
        }
    ) => {
        // Generate the trait
        gen_service_impl!(@generate_trait $dependencies, $($field_name : $field_type),*);

        // Then define the struct
        pub struct $service_name<Deps: $dependencies> {
            $(
                pub $field_attr: std::sync::Arc<Deps::$field_name>,
            )*
        }
    };

    // -----------------------------------------------------
    // 3) Extended pattern (WITH custom fields).
    //    Will match:
    //
    //    gen_service_impl! {
    //        struct ServiceName: some::Trait = MyDeps {
    //            Foo: foo::Foo = foo_field,
    //            Bar: bar::Bar = bar_field
    //        }
    //        ; custom_fields {
    //            some_string: String = my_string,
    //            some_number: i32 = my_i32
    //        }
    //    }
    //
    // -----------------------------------------------------
    (
        struct $service_name:ident : $trait:path = $dependencies:ident {
            $(
                $field_name:ident : $field_type:path = $field_attr:ident
            ),* $(,)?
        }
        ; custom_fields {
            $(
                $custom_field_name:ident : $custom_field_type:ty = $custom_field_attr:ident
            ),* $(,)?
        }
    ) => {
        // Generate the trait
        gen_service_impl!(@generate_trait $dependencies, $($field_name : $field_type),*);

        // Then define the struct, which now has both
        // the dependency fields (Arc<Deps::...>) and
        // the custom fields.
        pub struct $service_name<Deps: $dependencies> {
            $(
                pub $field_attr: std::sync::Arc<Deps::$field_name>,
            )*
            $(
                pub $custom_field_attr: $custom_field_type,
            )*
        }
    };
}
