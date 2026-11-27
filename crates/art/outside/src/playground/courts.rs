//! The games: the basketball hoop, the ping-pong table and the park chess table.

mod basketball_hoop;
mod park_chess_table;
mod ping_pong_table;

pub use basketball_hoop::basketball_hoop;
pub use park_chess_table::{park_chess_table, park_chess_table_far};
pub use ping_pong_table::ping_pong_table;
