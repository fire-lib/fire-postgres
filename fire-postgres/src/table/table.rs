use super::{Info, TableTemplate, ColumnData};
use super::util::{
	data_into_sql_params, rows_into_data, info_data_to_sql, quote
};

use crate::database::SharedClient;
use crate::query::{Query, UpdateParams, SqlBuilder};
use crate::Result;

use std::sync::Arc;
use std::marker::PhantomData;
use std::borrow::Borrow;
// use std::fmt::Write;

// use tokio_postgres::types::ToSql;
// use tokio_postgres::row::Row;

// is thread safe
// maybe should change to an inner?
macro_rules! debug_sql {
	($method:expr, $name:expr, $sql:expr) => (if cfg!(feature = "show-sql") {
		println!("sql: {} {} with {}", $method, $name, $sql);
	})
}

#[derive(Debug)]
struct TableMeta {
	info: Info,
	select: String,
	insert: String,
	update_full: SqlBuilder,
	names_for_select: String
}

#[derive(Debug)]
pub struct Table<T>
where T: TableTemplate {
	client: SharedClient,
	name: &'static str,
	meta: Arc<TableMeta>,
	phantom: PhantomData<T>
}

impl<T> Table<T>
where T: TableTemplate {

	pub(crate) fn new(client: SharedClient, name: &'static str) -> Self {
		let info = T::table_info();
		let meta = TableMeta {
			select: Self::create_select_sql(&info, name),
			insert: Self::create_insert_sql(&info, name),
			update_full: Self::create_update_full(&info),
			names_for_select: Self::create_names_for_select(&info),
			info
		};

		Self {
			client, name,
			meta: Arc::new(meta),
			phantom: PhantomData
		}
	}

	pub fn name(&self) -> &'static str {
		self.name
	}

	/// ## Example Output
	/// `"a", "b"`
	pub fn names_for_select(&self) -> &str {
		&self.meta.names_for_select
	}

	pub fn info(&self) -> &Info {
		&self.meta.info
	}

	fn create_names_for_select(info: &Info) -> String {
		format!("\"{}\"", info.names().join("\", \""))
	}

	fn create_select_sql(info: &Info, name: &str) -> String {
		let names = info.names();
		format!("SELECT \"{}\" FROM \"{}\"", names.join("\", \""), name)
	}

	fn create_insert_sql(info: &Info, name: &str) -> String {
		let mut names = vec![];
		let mut vals = vec![];
		for (i, col) in info.data().iter().enumerate() {
			names.push(quote(col.name));
			vals.push(format!("${}", i + 1));
		}

		// maybe could prepare basic sql already??
		format!(
			"INSERT INTO \"{}\" ({}) VALUES ({})",
			name,
			names.join(", "),
			vals.join(", ")
		)
	}

	// we need to return an SqlBuilder and not just a string is since
	// the where clause could also contain some parameters which would reset
	// the param counter
	fn create_update_full(info: &Info) -> SqlBuilder {
		let mut sql = SqlBuilder::new();

		let last = info.data().len() - 1;
		for (i, col) in info.data().iter().enumerate() {

			sql.space_after(format!("\"{}\" =", col.name));
			sql.param();

			if i != last {
				sql.space_after(",");
			}
		}

		sql
	}

	// Create
	pub async fn try_create(&self) -> Result<()> {

		let sql = info_data_to_sql(self.name, self.meta.info.data());

		debug_sql!("create", self.name, sql);

		self.client
			.read().await
			.batch_execute(sql.as_str()).await
			.map_err(Into::into)
	}

	/// ## Panics
	/// if the table could not be created
	pub async fn create(self) -> Self {
		self.try_create().await
			.expect("could not create table");
		self
	}



	/*pub async fn query_raw(
		&self,
		sql: &str,
		params: &[&(dyn ToSql + Sync)]
	) -> Result<Vec<Row>, PostgresError> {
		self.client.query(sql, params).await
	}

	pub async fn query_to_raw(
		&self,
		query: Query
	) -> Result<Vec<Row>, PostgresError> {
		let sql = query.sql().to_string();
		let data = query.to_sql_params();
		self.client.query(sql.as_str(), data.as_slice()).await
	}*/


	// find
	// maybe rename to insert
	// and store statement in table
	pub async fn insert_one(&self, input: &T) -> Result<()> {

		let sql = &self.meta.insert;
		debug_sql!("insert_one", self.name, sql);

		let cl = self.client.read().await;

		let data = input.to_data();
		let params = data_into_sql_params(&data);

		// don't use a prepare statement since this is executed only once
		cl.execute(sql, params.as_slice()).await?;
		Ok(())
	}

	pub async fn insert_many<B, I>(&self, input: I) -> Result<()>
	where
		B: Borrow<T>,
		I: Iterator<Item=B>
	{

		let sql = &self.meta.insert;
		debug_sql!("insert_many", self.name, sql);

		// we make a transaction so if an error should occur
		// we don't insert any data
		let mut cl = self.client.write().await;
		let ts = cl.transaction().await?;

		let stmt = ts.prepare(sql).await?;

		for input in input {
			let data = input.borrow().to_data();
			let params = data_into_sql_params(&data);

			ts.execute(&stmt, params.as_slice()).await?;
		}

		ts.commit().await?;

		Ok(())
	}


	/*
	SELECT id, name, FROM {}
	*/
	pub async fn find_all(&self) -> Result<Vec<T>> {

		let sql = &self.meta.select;
		debug_sql!("find_all", self.name, sql);

		let rows = {
			let cl = self.client.read().await;
			cl.query(sql, &[]).await?
		};

		rows_into_data(rows)
	}

	pub async fn find_many(&self, where_query: Query<'_>) -> Result<Vec<T>> {

		let mut query = Query::from_sql_str(self.meta.select.clone());

		self.meta.info.validate_params(where_query.params())?;
		query.sql.space("WHERE");
		query.append(where_query);

		let sql = query.sql().to_string();
		debug_sql!("find_many", self.name, sql);
		let params = query.to_sql_params();

		let rows = {
			let cl = self.client.read().await;
			cl.query(&sql, params.as_slice()).await?
		};

		rows_into_data(rows)
	}

	pub async fn find_one(
		&self,
		mut where_query: Query<'_>
	) -> Result<Option<T>> {
		where_query.sql.space_before("LIMIT 1");
		let res = self.find_many(where_query).await?;

		debug_assert!(res.len() <= 1);

		Ok(res.into_iter().next())
	}

	/// expects the rows to be in the order which get's returned by
	/// names_for_select
	pub async fn find_many_raw(&self, sql: &str) -> Result<Vec<T>> {
		debug_sql!("find_many_raw", self.name, sql);

		let rows = {
			let cl = self.client.read().await;
			cl.query(sql, &[]).await?
		};

		rows_into_data(rows)
	}

	// update one
	pub async fn update<'a>(
		&self,
		where_query: Query<'a>,
		update_query: UpdateParams<'a>
	) -> Result<()> {

		// UPDATE table SET column WHERE
		let mut query = update_query.into_query();
		query.sql.space("WHERE");
		query.append(where_query);

		self.meta.info.validate_params(query.params())?;

		let sql = format!(
			"UPDATE \"{}\" SET {}",
			self.name,
			query.sql().to_string()
		);
		debug_sql!("update", self.name, sql);
		let params = query.to_sql_params();

		let cl = self.client.read().await;
		cl.execute(&sql, params.as_slice()).await?;

		Ok(())
	}

	pub async fn update_full<'a>(
		&self,
		where_query: Query<'a>,
		input: &'a T
	) -> Result<()> {

		let mut sql = self.meta.update_full.clone();

		self.meta.info.validate_params(where_query.params())?;

		sql.space("WHERE");
		sql.append(where_query.sql);

		let sql = format!("UPDATE \"{}\" SET {}", self.name, sql.to_string());
		debug_sql!("update_full", self.name, sql);

		let mut data = input.to_data();
		for param in where_query.params {
			data.push(param.data);
		}
		let params = data_into_sql_params(&data);

		let cl = self.client.read().await;
		cl.execute(&sql, params.as_slice()).await?;

		Ok(())
	}

	// delete one
	pub async fn delete(&self, where_query: Query<'_>) -> Result<()> {

		self.meta.info.validate_params(where_query.params())?;

		let sql = format!(
			"DELETE FROM \"{}\" WHERE {}",
			self.name,
			where_query.sql.to_string()
		);
		debug_sql!("delete_many", self.name, sql);
		let params = where_query.to_sql_params();

		let cl = self.client.read().await;
		cl.execute(&sql, params.as_slice()).await?;

		Ok(())
	}

	/// this does not verify the params
	pub async fn execute_raw(
		&self,
		sql: SqlBuilder,
		data: &[ColumnData<'_>]
	) -> Result<()> {
		let sql = sql.to_string();
		debug_sql!("execute_raw", self.name, sql);

		let params = data_into_sql_params(data);

		let cl = self.client.read().await;
		cl.execute(&sql, params.as_slice()).await?;

		Ok(())
	}
}

impl<T> Clone for Table<T>
where T: TableTemplate {
	fn clone(&self) -> Self {
		Self {
			client: self.client.clone(),
			name: self.name,
			meta: self.meta.clone(),
			phantom: PhantomData
		}
	}
}