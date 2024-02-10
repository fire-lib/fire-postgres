use crate::table::column::{ColumnData, ColumnKind, ColumnType};

use std::borrow::Cow;
use std::fmt;

#[cfg(feature = "connect")]
use tokio_postgres::types::ToSql;

pub mod update;
pub mod whr;
pub use update::UpdateParams;

pub type SqlStr = Cow<'static, str>;

// find query
// select query
// insert query
// delete query
// update query

#[derive(Debug, Clone)]
enum SqlBuilderType {
	NoSpace(SqlStr),
	SpaceAfter(SqlStr),
	SpaceBefore(SqlStr),
	Space(SqlStr),
	Param,
}

#[derive(Debug, Clone)]
pub struct SqlBuilder {
	data: Vec<SqlBuilderType>,
}

impl SqlBuilder {
	pub fn new() -> Self {
		Self { data: vec![] }
	}

	pub fn from_sql_str(sql: impl Into<SqlStr>) -> Self {
		Self {
			data: vec![SqlBuilderType::SpaceAfter(sql.into())],
		}
	}

	pub fn no_space(&mut self, s: impl Into<SqlStr>) {
		self.data.push(SqlBuilderType::NoSpace(s.into()));
	}

	pub fn space_after(&mut self, s: impl Into<SqlStr>) {
		self.data.push(SqlBuilderType::SpaceAfter(s.into()));
	}

	pub fn space_before(&mut self, s: impl Into<SqlStr>) {
		self.data.push(SqlBuilderType::SpaceBefore(s.into()));
	}

	pub fn space(&mut self, s: impl Into<SqlStr>) {
		self.data.push(SqlBuilderType::Space(s.into()));
	}

	pub fn param(&mut self) {
		self.data.push(SqlBuilderType::Param);
	}

	pub fn prepend(&mut self, mut sql: SqlBuilder) {
		sql.data.append(&mut self.data);
		self.data = sql.data;
	}

	pub fn append(&mut self, mut sql: SqlBuilder) {
		self.data.append(&mut sql.data);
	}

	pub fn is_empty(&self) -> bool {
		self.data.is_empty()
	}
}

impl fmt::Display for SqlBuilder {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut c = 0;
		for d in &self.data {
			match d {
				SqlBuilderType::NoSpace(s) => {
					f.write_str(s)?;
				}
				SqlBuilderType::SpaceAfter(s) => {
					write!(f, "{} ", s)?;
				}
				SqlBuilderType::SpaceBefore(s) => {
					write!(f, " {}", s)?;
				}
				SqlBuilderType::Space(s) => {
					write!(f, " {} ", s)?;
				}
				SqlBuilderType::Param => {
					c += 1;
					write!(f, "${}", c)?;
				}
			}
		}

		Ok(())
	}
}

#[derive(Debug, Clone)]
pub struct Query<'a> {
	pub sql: SqlBuilder,
	pub params: Vec<Param<'a>>,
}

impl<'a> Query<'a> {
	pub fn new(sql: SqlBuilder, params: Vec<Param<'a>>) -> Self {
		Self { sql, params }
	}

	pub fn from_sql_str(sql: impl Into<SqlStr>) -> Self {
		Self {
			sql: SqlBuilder::from_sql_str(sql),
			params: vec![],
		}
	}

	pub fn prepend(&mut self, sql: SqlBuilder, mut params: Vec<Param<'a>>) {
		self.sql.prepend(sql);
		params.append(&mut self.params);
		self.params = params;
	}

	pub fn append(&mut self, mut query: Query<'a>) {
		self.sql.append(query.sql);
		self.params.append(&mut query.params);
	}

	pub fn append_raw(&mut self, sql: SqlBuilder, mut params: Vec<Param<'a>>) {
		self.sql.append(sql);
		self.params.append(&mut params);
	}

	pub fn sql(&self) -> &SqlBuilder {
		&self.sql
	}

	pub fn params(&self) -> &[Param] {
		self.params.as_slice()
	}

	pub fn is_empty(&self) -> bool {
		self.sql.is_empty() && self.params.is_empty()
	}

	pub fn params_data(&self) -> Vec<&ColumnData> {
		let mut v = Vec::with_capacity(self.params.len());
		for param in &self.params {
			v.push(param.data());
		}
		v
	}

	#[cfg(feature = "connect")]
	pub fn to_sql_params(&self) -> Vec<&(dyn ToSql + Sync)> {
		let mut v = Vec::with_capacity(self.params.len());
		for param in &self.params {
			v.push(param.data() as &(dyn ToSql + Sync));
		}
		v
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param<'a> {
	pub name: &'static str,
	pub kind: ColumnKind,
	pub data: ColumnData<'a>,
}

impl<'a> Param<'a> {
	pub fn new<T>(name: &'static str, data: &'a T) -> Self
	where
		T: ColumnType,
	{
		let kind = T::column_kind();
		Self {
			name,
			kind,
			data: data.to_data(),
		}
	}

	pub fn data(&self) -> &ColumnData {
		&self.data
	}

	#[inline(always)]
	pub fn maybe_null(&self) -> bool {
		matches!(self.kind, ColumnKind::Option(_))
	}
}

/*
find
- eq
- ne
- gt
- gte
- lt
- lte
- in
- nin

- and
- or
- not
- nor
*/
