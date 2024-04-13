pub mod info;
pub use info::Info;

pub mod column;

pub mod table;
pub use table::Table;

use crate::{
	row::{FromRowOwned, NamedColumns},
	update::ToUpdate,
};

mod util;

pub trait TableTemplate:
	FromRowOwned + NamedColumns + ToUpdate + Sized
{
	fn table_info() -> Info;
}
