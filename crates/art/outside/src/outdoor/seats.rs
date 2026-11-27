//! OU1's seats: the park bench, the folding bistro set, the picnic table, the garden and Adirondack chairs and
//! the three-seat garden bench.

mod bench;
mod chairs;
mod garden_bench;
mod patio_set;
mod picnic_table;

pub use bench::bench;
pub use chairs::{adirondack_chair, arm_board, garden_chair};
pub use garden_bench::bench_garden3;
pub use patio_set::patio_set;
pub use picnic_table::picnic_table;
