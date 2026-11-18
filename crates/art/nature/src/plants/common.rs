//! What the nature families' own helpers are written with.

/// A function's options, as artkit's own are written: a struct with defaults.
macro_rules! options {
    ($(#[$doc:meta])* $name:ident { $($field:ident : $ty:ty = $default:expr),* $(,)? }) => {
        $(#[$doc])*
        #[derive(Clone, Debug)]
        pub struct $name {
            $(pub $field: $ty,)*
        }
        impl Default for $name {
            fn default() -> $name {
                $name { $($field: $default,)* }
            }
        }
    };
}
pub(crate) use options;
