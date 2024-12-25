#[macro_export]
macro_rules! gen_service_impl {
    (
        struct $service_name:ident : $trait:path = $dependencies:ident {
            $($field_name:ident: $field_type:path = $field_attr:ident),*
        }
    ) => {
            pub trait $dependencies {
                type Context: Send + Sync + Clone + Eq + std::fmt::Debug + 'static;
                type Transaction: dao::Transaction + Send + Sync + Clone + std::fmt::Debug + 'static;
                $(
                    type $field_name: $field_type + Sync + Send;
                )*
            }

            pub struct $service_name<Deps: $dependencies> {
                $(
                    pub $field_attr: std::sync::Arc<Deps::$field_name>,
                )*
            }
    };
}
