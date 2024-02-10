#[derive(Debug, Clone, PartialEq)]
pub enum Text<'a> {
	Owned(String),
	Borrowed(&'a str),
}

impl Text<'_> {
	pub fn into_string(self) -> String {
		match self {
			Self::Owned(s) => s,
			Self::Borrowed(b) => b.to_string(),
		}
	}

	pub fn as_str(&self) -> &str {
		match self {
			Self::Owned(s) => s,
			Self::Borrowed(b) => b,
		}
	}

	pub fn len(&self) -> usize {
		match self {
			Self::Owned(s) => s.len(),
			Self::Borrowed(s) => s.len(),
		}
	}
}

impl<'a> From<&'a str> for Text<'a> {
	fn from(s: &'a str) -> Self {
		Self::Borrowed(s)
	}
}

impl From<String> for Text<'_> {
	fn from(s: String) -> Self {
		Self::Owned(s)
	}
}

//impl From<

pub enum SliceState<'a> {
	Owned(&'a [String]),
	Borrowed(&'a [&'a str]),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TextArray<'a> {
	SliceOwned(&'a [String]),
	SliceStr(&'a [&'a str]),
	VecOwned(Vec<String>),
	VecStr(Vec<&'a str>),
}

impl<'a> TextArray<'a> {
	pub fn len(&self) -> usize {
		match self {
			Self::SliceOwned(v) => v.len(),
			Self::SliceStr(v) => v.len(),
			Self::VecOwned(v) => v.len(),
			Self::VecStr(v) => v.len(),
		}
	}

	pub fn into_vec_owned(self) -> Vec<String> {
		match self {
			Self::SliceOwned(s) => s.iter().map(|v| v.to_string()).collect(),
			Self::SliceStr(s) => s.iter().map(|v| v.to_string()).collect(),
			Self::VecOwned(v) => v,
			Self::VecStr(v) => v.iter().map(|v| v.to_string()).collect(),
		}
	}

	/*pub fn as_slice_str(&self) -> Vec<&str> {
		match self {
			Self::SliceOwned(s) => s.iter(),
			Self::SliceStr(s) => s.iter().collect(),
			Self::VecOwned(s) => s.iter().collect(),
			Self::VecStr(s) => s
		}
	}*/

	/*pub fn to_vec_str(self) -> Vec<&'a str> {
		match self {
			Self::SliceOwned(s) => s.iter().map(|v| v.as_str()).collect(),
			Self::SliceStr(s) => s.iter().map(|v| *v).collect(),
			Self::VecOwned(s) => s.into_iter().map(|v| v.as_str()).collect(),
			Self::VecStr(v) => v
		}
	}*/

	pub fn unwrap_vec_str(self) -> Vec<&'a str> {
		match self {
			Self::VecStr(v) => v,
			_ => panic!("could not unwrap vec str from textarray"),
		}
	}

	/*pub fn try_slice_owned(&self) -> Option<&[String]> {
		Some(match self {
			Self::SliceOwned(v) => v,
			Self::VecOwned(v) => v.as_slice(),
			_ => return None
		})
	}*/

	pub fn to_slice_state(&self) -> SliceState<'_> {
		match self {
			Self::SliceOwned(v) => SliceState::Owned(v),
			Self::SliceStr(v) => SliceState::Borrowed(v),
			Self::VecOwned(v) => SliceState::Owned(v.as_slice()),
			Self::VecStr(v) => SliceState::Borrowed(v.as_slice()),
		}
	}
}

impl<'a> From<&'a [String]> for TextArray<'a> {
	fn from(ar: &'a [String]) -> Self {
		Self::SliceOwned(ar)
	}
}

impl<'a> From<&'a [&'a str]> for TextArray<'a> {
	fn from(ar: &'a [&'a str]) -> Self {
		Self::SliceStr(ar)
	}
}

impl From<Vec<String>> for TextArray<'_> {
	fn from(ar: Vec<String>) -> Self {
		Self::VecOwned(ar)
	}
}

impl<'a> From<Vec<&'a str>> for TextArray<'a> {
	fn from(ar: Vec<&'a str>) -> Self {
		Self::VecStr(ar)
	}
}

/*impl TextArray {
	pub fn next() -> Result<Option<Text>, FromDataError> {

	}
}*/

// Owned and referenced??
#[derive(Debug, Clone, PartialEq)]
pub enum ColumnData<'a> {
	Boolean(bool),
	Text(Text<'a>),
	Date(i32),
	Timestamp(i64),
	F64(f64),
	F32(f32),
	I64(i64),
	I32(i32),
	I16(i16),
	Option(Option<Box<ColumnData<'a>>>),
	TextArray(TextArray<'a>),
	Bytea(&'a [u8]),
}

impl<'a> ColumnData<'a> {
	/*pub fn text(s: &'static str) -> Self {
		Self::Text(s)
	}

	pub fn unwrap_text(self) -> &'a str {
		match self {
			Self::Text(t) => t,
			_ => panic!("could not unwrap text")
		}
	}*/

	#[inline(always)]
	pub fn is_null(&self) -> bool {
		matches!(self, Self::Option(None))
	}

	pub fn unwrap_text(self) -> Text<'a> {
		match self {
			Self::Text(t) => t,
			_ => panic!("could not unwrap text"),
		}
	}
}

#[cfg(feature = "connect")]
mod impl_postgres {

	use super::*;

	use std::convert::TryFrom;
	use std::error::Error;

	use bytes::BytesMut;

	use fallible_iterator::FallibleIterator;

	use postgres_types::{accepts, to_sql_checked};
	use postgres_types::{FromSql, IsNull, Kind as PostgresKind, ToSql, Type};

	use postgres_protocol::types as ty;
	use postgres_protocol::IsNull as ProtIsNull;

	type BoxedError = Box<dyn Error + Send + Sync + 'static>;

	macro_rules! accepts {
		() => {
			fn accepts(ty: &Type) -> bool {
				match ty {
					&Type::BOOL
					| &Type::BPCHAR
					| &Type::VARCHAR
					| &Type::TEXT
					| &Type::DATE
					| &Type::TIMESTAMP
					| &Type::FLOAT8
					| &Type::FLOAT4
					| &Type::INT8
					| &Type::INT4
					| &Type::INT2
					| &Type::BYTEA
					| &Type::JSON => true,
					t => match t.kind() {
						PostgresKind::Array(Type::TEXT) => true,
						_ => false,
					},
				}
			}
		};
	}

	fn text_array_to_sql<'a, I, T, F>(
		len: usize,
		elements: I,
		f: F,
		buf: &mut BytesMut,
	) -> Result<(), BoxedError>
	where
		I: IntoIterator<Item = T>,
		F: Fn(T) -> &'a str,
	{
		let len = i32::try_from(len).map_err(|_| "cannot convert i16 to u8")?;
		let dimension = ty::ArrayDimension {
			len,
			lower_bound: 1,
		};

		let text_oid = Type::TEXT.oid();

		ty::array_to_sql(
			Some(dimension),
			text_oid,
			elements,
			|e, buf| {
				ty::text_to_sql(f(e), buf);
				Ok(ProtIsNull::No)
			},
			buf,
		)
	}

	impl<'a> ToSql for ColumnData<'a> {
		fn to_sql(
			&self,
			_ty: &Type,
			out: &mut BytesMut,
		) -> Result<IsNull, BoxedError> {
			match self {
				ColumnData::Boolean(v) => ty::bool_to_sql(*v, out),
				ColumnData::Text(v) => ty::text_to_sql(v.as_str(), out),
				ColumnData::Date(v) => ty::date_to_sql(*v, out),
				ColumnData::Timestamp(v) => ty::timestamp_to_sql(*v, out),
				ColumnData::F64(v) => ty::float8_to_sql(*v, out),
				ColumnData::F32(v) => ty::float4_to_sql(*v, out),
				ColumnData::I64(v) => ty::int8_to_sql(*v, out),
				ColumnData::I32(v) => ty::int4_to_sql(*v, out),
				ColumnData::I16(v) => ty::int2_to_sql(*v, out),
				ColumnData::Option(o) => match o {
					Some(v) => return v.to_sql(_ty, out),
					None => return Ok(IsNull::Yes),
				},
				ColumnData::Bytea(v) => ty::bytea_to_sql(v, out),
				ColumnData::TextArray(v) => match v.to_slice_state() {
					SliceState::Owned(o) => text_array_to_sql(
						o.len(),
						o.iter(),
						|v| v.as_str(),
						out,
					)?,
					SliceState::Borrowed(b) => {
						text_array_to_sql(b.len(), b.iter(), |v| v, out)?
					}
				},
				//ColumnData::TextArray(v) => text_array_to_sql(v.len(), v, out)?,
				//ColumnData::TextVecString(v) => text_array_to_sql(v.len(), v, out)?,
			};
			Ok(IsNull::No)
		}

		accepts!();

		to_sql_checked!();
	}

	impl<'a> FromSql<'a> for ColumnData<'a> {
		fn from_sql(ty: &Type, raw: &'a [u8]) -> Result<Self, BoxedError> {
			Ok(match ty {
				&Type::BOOL => Self::Boolean(ty::bool_from_sql(raw)?),
				&Type::BPCHAR | &Type::VARCHAR | &Type::TEXT | &Type::JSON => {
					Self::Text(ty::text_from_sql(raw)?.into())
				}
				&Type::DATE => Self::Date(ty::date_from_sql(raw)?),
				&Type::TIMESTAMP => {
					Self::Timestamp(ty::timestamp_from_sql(raw)?)
				}
				&Type::FLOAT8 => Self::F64(ty::float8_from_sql(raw)?),
				&Type::FLOAT4 => Self::F32(ty::float4_from_sql(raw)?),
				&Type::INT8 => Self::I64(ty::int8_from_sql(raw)?),
				&Type::INT4 => Self::I32(ty::int4_from_sql(raw)?),
				&Type::INT2 => Self::I16(ty::int2_from_sql(raw)?),
				&Type::BYTEA => Self::Bytea(ty::bytea_from_sql(raw)),
				// &Type::TEXTARRAY
				t => {
					match t.kind() {
						PostgresKind::Array(Type::TEXT) => {}
						_ => return Err("type not recognized".into()),
					};

					let array = ty::array_from_sql(raw)?;
					if array.dimensions().count()? > 1 {
						return Err("array contains too many dimensions".into());
					}

					if array.element_type() != Type::TEXT.oid() {
						return Err(
							"expected array with TEXT AS Element".into()
						);
					}

					let mut values = vec![];
					let mut array = array.values();
					while let Some(buf) = array.next()? {
						match buf {
							Some(buf) => values.push(ty::text_from_sql(buf)?),
							None => {
								return Err("array items cannot be null".into())
							}
						}
					}

					Self::TextArray(values.into())
				}
			})
		}

		fn from_sql_null(
			_: &Type,
		) -> Result<Self, Box<dyn Error + Sync + Send>> {
			Ok(Self::Option(None))
		}

		accepts!();
	}
}
