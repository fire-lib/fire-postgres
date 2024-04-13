use std::borrow::Cow;
use std::fmt;

use tokio_postgres::types::ToSql;
use types::time::{Date, DateTime, Timeout};
use types::uid::UniqueId;

// pub mod update;
mod whr;
// pub use update::UpdateParams;

pub type SqlStr = Cow<'static, str>;

// find query
// select query
// insert query
// delete query
// update query

/// (Where, OrderBy, Limit, Offset)

#[derive(Debug)]
#[non_exhaustive]
pub struct Filter<'a> {
	pub whr: Where,
	pub order_by: OrderBy,
	pub limit: Limit,
	pub offset: Offset,
	pub params: Params<'a>,
}

impl<'a> Filter<'a> {
	pub fn new() -> Self {
		Self {
			whr: Where::new(),
			order_by: OrderBy::new(),
			limit: Limit::new(),
			offset: Offset::new(),
			params: Params::new(),
		}
	}

	pub(crate) fn to_formatter(&'a self) -> FilterFormatter<'a> {
		FilterFormatter {
			whr: &self.whr,
			order_by: &self.order_by,
			limit: &self.limit,
			offset: &self.offset,
			params: &self.params,
		}
	}
}

impl fmt::Display for Filter<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.to_formatter())
	}
}

#[derive(Debug)]
#[non_exhaustive]
pub(crate) struct FilterFormatter<'a> {
	pub whr: &'a Where,
	pub order_by: &'a OrderBy,
	pub limit: &'a Limit,
	pub offset: &'a Offset,
	pub params: &'a Params<'a>,
}

impl fmt::Display for FilterFormatter<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if !self.whr.is_empty() {
			write!(f, " WHERE {}", self.whr)?;
		}

		if !self.order_by.is_empty() {
			write!(f, " ORDER BY {}", self.order_by)?;
		}

		let offset_has_param = matches!(self.offset, Offset::Param);
		let param_count =
			self.params.len() - if offset_has_param { 1 } else { 0 };

		match &self.limit {
			Limit::Fixed(value) => write!(f, " LIMIT {}", value)?,
			Limit::Param => {
				write!(f, " LIMIT ${}", param_count)?;
			}
			Limit::All => {}
		}

		match &self.offset {
			Offset::Zero => {}
			Offset::Fixed(value) => write!(f, " OFFSET {}", value)?,
			Offset::Param => {
				write!(f, " OFFSET ${}", self.params.len())?;
			}
		}

		Ok(())
	}
}

#[derive(Debug)]
#[non_exhaustive]
pub struct WhereFilter<'a> {
	pub whr: Where,
	pub params: Params<'a>,
}

impl<'a> WhereFilter<'a> {
	pub fn new() -> Self {
		Self {
			whr: Where::new(),
			params: Params::new(),
		}
	}
}

impl fmt::Display for WhereFilter<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if !self.whr.is_empty() {
			write!(f, " WHERE {}", self.whr)?;
		}

		Ok(())
	}
}

#[derive(Debug)]
pub struct Where {
	inner: Vec<WherePart>,
}

#[derive(Debug)]
pub enum WherePart {
	Operation(WhereOperation),
	And,
	Or,
}

#[derive(Debug)]
pub struct WhereOperation {
	pub kind: Operator,
	pub column: Cow<'static, str>,
}

#[derive(Debug)]
pub enum Operator {
	Eq,
	Ne,
	Lt,
	Lte,
	Gt,
	Gte,
	Like,
	In,

	// rhs will be ignored
	IsNull,
	// rhs will be ignored
	IsNotNull,
}

#[derive(Debug)]
pub enum WhereIdent {
	Param,
	Name(Cow<'static, str>),
}

impl Where {
	pub fn new() -> Self {
		Self { inner: vec![] }
	}

	pub fn push(&mut self, part: impl Into<WherePart>) {
		self.inner.push(part.into());
	}

	fn is_empty(&self) -> bool {
		self.inner.is_empty()
	}
}

impl fmt::Display for Where {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut param_num = 0;

		for part in &self.inner {
			match part {
				WherePart::And => f.write_str(" AND ")?,
				WherePart::Or => f.write_str(" OR ")?,
				WherePart::Operation(op) => match &op.kind {
					Operator::IsNull | Operator::IsNotNull => {
						write!(f, "\"{}\" {}", op.column, op.kind.as_str())?;
					}
					o => {
						param_num += 1;

						write!(
							f,
							"\"{}\" {} ${}",
							op.column,
							o.as_str(),
							param_num
						)?;
					}
				},
			}
		}

		Ok(())
	}
}

impl From<WhereOperation> for WherePart {
	fn from(op: WhereOperation) -> Self {
		Self::Operation(op)
	}
}

impl Operator {
	fn as_str(&self) -> &str {
		match self {
			Operator::Eq => "=",
			Operator::Ne => "!=",
			Operator::Lt => "<",
			Operator::Lte => "<=",
			Operator::Gt => ">",
			Operator::Gte => ">=",
			Operator::Like => "LIKE",
			Operator::In => "IN",
			Operator::IsNull => "IS NULL",
			Operator::IsNotNull => "IS NOT NULL",
		}
	}
}

#[derive(Debug)]
pub struct OrderBy {
	inner: Vec<OrderByPart>,
}

#[derive(Debug)]
pub enum OrderByPart {
	Asc(Cow<'static, str>),
	Desc(Cow<'static, str>),
}

impl OrderBy {
	pub fn new() -> Self {
		Self { inner: vec![] }
	}

	pub fn push_asc(&mut self, column: impl Into<Cow<'static, str>>) {
		self.inner.push(OrderByPart::Asc(column.into()));
	}

	pub fn push_desc(&mut self, column: impl Into<Cow<'static, str>>) {
		self.inner.push(OrderByPart::Desc(column.into()));
	}

	fn is_empty(&self) -> bool {
		self.inner.is_empty()
	}
}

impl fmt::Display for OrderBy {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		for (i, part) in self.inner.iter().enumerate() {
			if i != 0 {
				f.write_str(", ")?;
			}

			match part {
				OrderByPart::Asc(column) => write!(f, "\"{}\" ASC", column)?,
				OrderByPart::Desc(column) => write!(f, "\"{}\" DESC", column)?,
			}
		}

		Ok(())
	}
}

#[derive(Debug)]
pub enum Limit {
	Fixed(usize),
	Param,
	All,
}

impl Limit {
	pub fn new() -> Self {
		Self::All
	}

	pub fn set_param(&mut self) {
		*self = Self::Param;
	}

	pub fn set_fixed(&mut self, value: usize) {
		*self = Self::Fixed(value);
	}
}

#[derive(Debug)]
pub enum Offset {
	Zero,
	Fixed(usize),
	Param,
}

impl Offset {
	pub fn new() -> Self {
		Self::Zero
	}

	pub fn set_param(&mut self) {
		*self = Self::Param;
	}

	pub fn set_fixed(&mut self, value: usize) {
		*self = Self::Fixed(value);
	}
}

// #[derive(Debug, Clone)]
// enum SqlBuilderType {
// 	NoSpace(SqlStr),
// 	SpaceAfter(SqlStr),
// 	SpaceBefore(SqlStr),
// 	Space(SqlStr),
// 	Param,
// }

// #[derive(Debug, Clone)]
// pub struct SqlBuilder {
// 	data: Vec<SqlBuilderType>,
// }

// impl SqlBuilder {
// 	pub fn new() -> Self {
// 		Self { data: vec![] }
// 	}

// 	pub fn from_sql_str(sql: impl Into<SqlStr>) -> Self {
// 		Self {
// 			data: vec![SqlBuilderType::SpaceAfter(sql.into())],
// 		}
// 	}

// 	pub fn no_space(&mut self, s: impl Into<SqlStr>) {
// 		self.data.push(SqlBuilderType::NoSpace(s.into()));
// 	}

// 	pub fn space_after(&mut self, s: impl Into<SqlStr>) {
// 		self.data.push(SqlBuilderType::SpaceAfter(s.into()));
// 	}

// 	pub fn space_before(&mut self, s: impl Into<SqlStr>) {
// 		self.data.push(SqlBuilderType::SpaceBefore(s.into()));
// 	}

// 	pub fn space(&mut self, s: impl Into<SqlStr>) {
// 		self.data.push(SqlBuilderType::Space(s.into()));
// 	}

// 	pub fn param(&mut self) {
// 		self.data.push(SqlBuilderType::Param);
// 	}

// 	pub fn prepend(&mut self, mut sql: SqlBuilder) {
// 		sql.data.append(&mut self.data);
// 		self.data = sql.data;
// 	}

// 	pub fn append(&mut self, mut sql: SqlBuilder) {
// 		self.data.append(&mut sql.data);
// 	}

// 	pub fn is_empty(&self) -> bool {
// 		self.data.is_empty()
// 	}
// }

// impl fmt::Display for SqlBuilder {
// 	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
// 		let mut c = 0;
// 		for d in &self.data {
// 			match d {
// 				SqlBuilderType::NoSpace(s) => {
// 					f.write_str(s)?;
// 				}
// 				SqlBuilderType::SpaceAfter(s) => {
// 					write!(f, "{} ", s)?;
// 				}
// 				SqlBuilderType::SpaceBefore(s) => {
// 					write!(f, " {}", s)?;
// 				}
// 				SqlBuilderType::Space(s) => {
// 					write!(f, " {} ", s)?;
// 				}
// 				SqlBuilderType::Param => {
// 					c += 1;
// 					write!(f, "${}", c)?;
// 				}
// 			}
// 		}

// 		Ok(())
// 	}
// }

// #[derive(Debug, Clone)]
// pub struct Query<'a> {
// 	pub sql: SqlBuilder,
// 	pub params: Vec<Param<'a>>,
// }

// impl<'a> Query<'a> {
// 	pub fn new(sql: SqlBuilder, params: Vec<Param<'a>>) -> Self {
// 		Self { sql, params }
// 	}

// 	pub fn from_sql_str(sql: impl Into<SqlStr>) -> Self {
// 		Self {
// 			sql: SqlBuilder::from_sql_str(sql),
// 			params: vec![],
// 		}
// 	}

// 	pub fn prepend(&mut self, sql: SqlBuilder, mut params: Vec<Param<'a>>) {
// 		self.sql.prepend(sql);
// 		params.append(&mut self.params);
// 		self.params = params;
// 	}

// 	pub fn append(&mut self, mut query: Query<'a>) {
// 		self.sql.append(query.sql);
// 		self.params.append(&mut query.params);
// 	}

// 	pub fn append_raw(&mut self, sql: SqlBuilder, mut params: Vec<Param<'a>>) {
// 		self.sql.append(sql);
// 		self.params.append(&mut params);
// 	}

// 	pub fn sql(&self) -> &SqlBuilder {
// 		&self.sql
// 	}

// 	pub fn params(&self) -> &[Param] {
// 		self.params.as_slice()
// 	}

// 	pub fn is_empty(&self) -> bool {
// 		self.sql.is_empty() && self.params.is_empty()
// 	}

// 	// pub fn params_data(&self) -> Vec<&ColumnData> {
// 	// 	let mut v = Vec::with_capacity(self.params.len());
// 	// 	for param in &self.params {
// 	// 		v.push(param.data());
// 	// 	}
// 	// 	v
// 	// }

// 	// #[cfg(feature = "connect")]
// 	pub fn to_sql_params(&self) -> Vec<&(dyn ToSql + Sync)> {
// 		let mut v = Vec::with_capacity(self.params.len());
// 		for param in &self.params {
// 			v.push(param.data() as &(dyn ToSql + Sync));
// 		}
// 		v
// 	}
// }

#[derive(Debug)]
pub struct Params<'a> {
	inner: Vec<Param<'a>>,
}

impl<'a> Params<'a> {
	pub fn new() -> Self {
		Self { inner: vec![] }
	}

	pub fn push(&mut self, param: Param<'a>) {
		self.inner.push(param);
	}

	pub fn len(&self) -> usize {
		self.inner.len()
	}

	pub fn is_empty(&self) -> bool {
		self.inner.is_empty()
	}

	pub fn iter_to_sql(
		&self,
	) -> impl ExactSizeIterator<Item = &(dyn ToSql + Sync)> {
		self.inner.iter().map(|p| p.data.as_ref())
	}
}

#[derive(Debug)]
#[non_exhaustive]
pub struct Param<'a> {
	pub name: &'static str,
	// pub kind: ColumnKind,
	pub data: CowParamData<'a>,
	is_null: bool,
}

impl<'a> Param<'a> {
	pub fn new<T>(name: &'static str, data: &'a T) -> Self
	where
		T: ParamData + ToSql + Sync,
	{
		Self {
			name,
			is_null: data.is_null(),
			data: CowParamData::Borrowed(data),
		}
	}

	pub fn new_owned<T>(name: &'static str, data: T) -> Self
	where
		T: ParamData + ToSql + Sync + 'static,
	{
		Self {
			name,
			is_null: data.is_null(),
			data: CowParamData::Owned(Box::new(data)),
		}
	}

	// pub fn data(&self) -> &ColumnData {
	// 	&self.data
	// }

	// #[inline(always)]
	// pub fn maybe_null(&self) -> bool {
	// 	matches!(self.kind, ColumnKind::Option(_))
	// }

	pub fn is_null(&self) -> bool {
		self.is_null
	}
}

#[derive(Debug)]
pub enum CowParamData<'a> {
	Borrowed(&'a (dyn ToSql + Sync)),
	Owned(Box<dyn ToSql + Sync>),
}

impl<'a> CowParamData<'a> {
	pub fn as_ref(&self) -> &(dyn ToSql + Sync) {
		match self {
			CowParamData::Borrowed(data) => *data,
			CowParamData::Owned(data) => &**data,
		}
	}
}

pub trait ParamData {
	fn is_null(&self) -> bool;
}

impl<T> ParamData for &T
where
	T: ParamData + ?Sized,
{
	fn is_null(&self) -> bool {
		(**self).is_null()
	}
}

impl<T> ParamData for &mut T
where
	T: ParamData + ?Sized,
{
	fn is_null(&self) -> bool {
		(**self).is_null()
	}
}

impl<T> ParamData for Option<T> {
	fn is_null(&self) -> bool {
		self.is_none()
	}
}

impl<T> ParamData for Vec<T> {
	fn is_null(&self) -> bool {
		// todo should an empty array be considered null?
		false
	}
}

impl<T> ParamData for [T] {
	fn is_null(&self) -> bool {
		// todo should an empty array be considered null?
		false
	}
}

impl<'a, T> ParamData for Cow<'a, T>
where
	T: ToOwned + ParamData,
{
	fn is_null(&self) -> bool {
		(**self).is_null()
	}
}

#[macro_export]
macro_rules! param_not_null {
	($impl_for:ty) => {
		impl ParamData for $impl_for {
			fn is_null(&self) -> bool {
				false
			}
		}
	};

	($( $impl_for:ty ),*) => {
		$(
			$crate::param_not_null!($impl_for);
		)*
	};
}

param_not_null!(
	String, str, bool, f64, f32, i64, i32, i16, i8, UniqueId, DateTime, Date,
	Timeout
);

#[cfg(feature = "email")]
impl ParamData for email_address::EmailAddress {
	fn is_null(&self) -> bool {
		false
	}
}

// #[derive(Debug)]
// pub struct LikeString<T> {
// 	pub kind: LikeKind,
// 	pub inner: T,
// }

// #[derive(Debug)]
// pub enum LikeKind {
// 	LeftRight,
// 	Left,
// 	Right,
// }

// impl<T> LikeString<T> {
// 	pub fn left_right(inner: T) -> Self {
// 		Self {
// 			kind: LikeKind::LeftRight,
// 			inner,
// 		}
// 	}
// }

// impl<T> ToSql for LikeString<T>
// where
// 	T: fmt::Display + fmt::Debug,
// {
// 	fn to_sql(
// 		&self,
// 		ty: &Type,
// 		w: &mut bytes::BytesMut,
// 	) -> Result<IsNull, Box<dyn std::error::Error + Sync + Send>> {
// 		let s = match self.kind {
// 			LikeKind::LeftRight => format!("%{}%", self.inner),
// 			LikeKind::Left => format!("%{}", self.inner),
// 			LikeKind::Right => format!("{}%", self.inner),
// 		};

// 		ToSql::to_sql(&s, ty, w)
// 	}

// 	fn accepts(ty: &Type) -> bool {
// 		<&str as ToSql>::accepts(ty)
// 	}

// 	to_sql_checked!();
// }
