pub mod info;
pub use info::Info;

pub mod column;

pub mod table_owned;
pub use table_owned::TableOwned;

pub mod table;
pub use table::Table;

use crate::row::{FromRowOwned, NamedColumns, ToRow};

mod util;

pub trait TableTemplate: FromRowOwned + NamedColumns + ToRow + Sized {
	fn table_info() -> Info;
}
