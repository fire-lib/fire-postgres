use fire_postgres::row::NamedColumns;
use fire_postgres::update::ToUpdate;
use fire_postgres::{FromRow, TableTempl, ToUpdate, UniqueId};

#[derive(Debug, TableTempl, FromRow, ToUpdate)]
pub struct Table {
	#[index(primary)]
	pub id: UniqueId,
	pub name: Option<String>,
	pub age: i32,
}

#[derive(Debug, FromRow)]
#[allow(dead_code)]
pub struct Count(u32);

#[test]
fn test_select_columns() {
	assert_eq!(Table::select_columns(), r#""id", "name", "age""#);
}

#[test]
fn test_insert_columns() {
	assert_eq!(Table::insert_columns(), r#""id", "name", "age""#);
}
