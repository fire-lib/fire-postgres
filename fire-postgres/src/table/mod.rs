pub mod info;
pub use info::Info;

pub mod column;
use column::{ColumnData, FromDataError};

// pub mod derive;

#[cfg(feature = "connect")]
pub mod table;
#[cfg(feature = "connect")]
pub use table::Table;

#[cfg(feature = "connect")]
mod util;

// should add serialize and deserialize function???
pub trait TableTemplate: Sized {
	fn table_info() -> Info;

	fn to_data(&self) -> Vec<ColumnData<'_>>;

	fn from_data(data: Vec<ColumnData>) -> Result<Self, FromDataError>;
}
