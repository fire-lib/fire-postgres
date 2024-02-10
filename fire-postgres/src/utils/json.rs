use std::ops;

use serde::Serialize;

use crate::table::column::{ColumnData, ColumnKind, ColumnType, FromDataError};

#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// A type that holds a value which can be serialized or deserialized to json
pub struct Json<T>(pub T);

impl<T> ops::Deref for Json<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<T> From<T> for Json<T> {
	fn from(t: T) -> Self {
		Self(t)
	}
}

impl<T> Json<T> {
	pub fn into_inner(self) -> T {
		self.0
	}
}

impl<T> ColumnType for Json<T>
where
	T: Serialize + for<'de> serde::Deserialize<'de>,
{
	fn column_kind() -> ColumnKind {
		ColumnKind::Json
	}

	fn to_data(&self) -> ColumnData<'_> {
		let s = serde_json::to_string(&self.0)
			.expect(&format!("could not serialize {}", stringify!($struct)));
		ColumnData::Text(s.into())
	}

	fn from_data(data: ColumnData) -> std::result::Result<Self, FromDataError> {
		match data {
			ColumnData::Text(s) => serde_json::from_str(s.as_str())
				.map(Self)
				.map_err(|e| FromDataError::CustomString(e.to_string())),
			_ => Err(FromDataError::ExpectedType("str for json")),
		}
	}
}
