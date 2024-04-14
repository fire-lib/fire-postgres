// macro internal
#[doc(hidden)]
pub use postgres_types::ToSql;

pub trait ToRow {
	/// should return something like "id", "name", "email"
	fn insert_columns() -> &'static str;
	/// should return something like $1, $2, $3
	fn insert_values() -> &'static str;
	/// should return  something like "id" = $1, "name" = $2
	fn update_columns() -> &'static str;

	fn params_len() -> usize;
	fn params(&self) -> impl ExactSizeIterator<Item = &(dyn ToSql + Sync)>;
}
