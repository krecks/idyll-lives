//! The game's item models, generated: kitchens, bathrooms and the laundry. One folder a family, each
//! listed in `families()`.

use artkit::family::Family;

/// The kitchen's and the cookers' `pull` as their scripts called it: `pull!(F, name, a, b, face, out = .., r = None,
/// ...)`. Without a material it is the chrome of the family that calls it (its own `M`).
macro_rules! pull {
    ($f:expr, $name:expr, $a:expr, $b:expr, $face:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        #[allow(unused_mut)]
        let mut o = $crate::common::Pull::default();
        $(o.$k = $v;)*
        $crate::common::pull($f, $name, $a, $b, $face, o.material.unwrap_or(M.chrome), o.out, o.w, o.t, o.r, o.verts)
    }};
}

mod common;

pub mod bathroom;
pub mod kitchen;
pub mod kitchen_cooking;
pub mod kitchen_counters;
pub mod laundry;

/// The group's families by their names on the command line (`cargo art <name>`).
pub fn families() -> Vec<(&'static str, Family)> {
    vec![
        ("kitchen", kitchen::family()),
        ("kitchen_cooking", kitchen_cooking::family()),
        ("kitchen_counters", kitchen_counters::family()),
        ("bathroom", bathroom::family()),
        ("laundry", laundry::family()),
    ]
}
