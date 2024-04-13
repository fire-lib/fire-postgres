use postgres_types::ToSql;

pub trait ToUpdate {
	/// should return something like "id", "name", "email"
	fn insert_columns() -> &'static str;
	/// should return something like $1, $2, $3
	fn insert_values() -> &'static str;

	fn update_columns() -> &'static str;

	fn params_len() -> usize;
	fn params(&self) -> &[&(dyn ToSql + Sync)];
}
