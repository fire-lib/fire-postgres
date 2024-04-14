mod from;
mod to;

use std::{
	error::Error as StdError,
	pin::Pin,
	task::{Context, Poll},
};

use futures_util::Stream;
use pin_project_lite::pin_project;
use postgres_types::FromSql;
use tokio_postgres::row::RowIndex;
pub use tokio_postgres::Column;

use crate::connection::Error;

pub use from::{FromRow, FromRowOwned};
pub use to::ToRow;

pub trait NamedColumns {
	/// should return something like "id", "name", "email"
	fn select_columns() -> &'static str;
}

#[derive(Debug)]
#[repr(transparent)]
pub struct Row {
	row: tokio_postgres::Row,
}

impl Row {
	/// Returns information about the columns of data in the row.
	pub fn columns(&self) -> &[Column] {
		self.row.columns()
	}

	/// Determines if the row contains no values.
	pub fn is_empty(&self) -> bool {
		self.row.is_empty()
	}

	/// Returns the number of values in the row.
	pub fn len(&self) -> usize {
		self.row.len()
	}

	/// Deserializes the row.
	pub fn deserialize<'a, T>(
		&'a self,
	) -> Result<T, Box<dyn StdError + Sync + Send>>
	where
		T: FromRow<'a>,
	{
		T::from_row(self)
	}

	/// Deserializes the row and consumes it.
	///
	/// todo or deserialize_into?
	pub fn deserialize_owned<T>(
		self,
	) -> Result<T, Box<dyn StdError + Sync + Send>>
	where
		T: FromRowOwned,
	{
		T::from_row_owned(self)
	}

	/// Deserializes a value from the row.
	///
	/// The value can be specified either by its numeric index in the row, or by its column name.
	///
	/// # Panics
	///
	/// Panics if the index is out of bounds or if the value cannot be converted to the specified type.
	pub fn get<'a, I, T>(&'a self, idx: I) -> T
	where
		I: RowIndex + std::fmt::Display,
		T: FromSql<'a>,
	{
		self.row.get(idx)
	}

	/// Like [`Row::get()`], but returns a [`Result`] rather than panicking.
	pub fn try_get<'a, I, T>(
		&'a self,
		idx: I,
	) -> Result<T, tokio_postgres::Error>
	where
		I: RowIndex + std::fmt::Display,
		T: FromSql<'a>,
	{
		self.row.try_get(idx)
	}
}

impl From<tokio_postgres::Row> for Row {
	fn from(row: tokio_postgres::Row) -> Self {
		Self { row }
	}
}

impl FromRowOwned for Row {
	fn from_row_owned(
		row: Row,
	) -> Result<Self, Box<dyn StdError + Sync + Send>> {
		Ok(row)
	}
}

pin_project! {
	pub struct RowStream {
		#[pin]
		inner: tokio_postgres::RowStream,
	}
}

impl Stream for RowStream {
	type Item = Result<Row, Error>;

	fn poll_next(
		self: Pin<&mut Self>,
		cx: &mut Context<'_>,
	) -> Poll<Option<Self::Item>> {
		let this = self.project();

		match this.inner.poll_next(cx) {
			Poll::Ready(Some(Ok(row))) => Poll::Ready(Some(Ok(row.into()))),
			Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e.into()))),
			Poll::Ready(None) => Poll::Ready(None),
			Poll::Pending => Poll::Pending,
		}
	}
}

impl From<tokio_postgres::RowStream> for RowStream {
	fn from(inner: tokio_postgres::RowStream) -> Self {
		Self { inner }
	}
}
