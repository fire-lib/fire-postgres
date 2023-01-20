use super::{ColumnKind, ColumnData};

use std::convert::TryFrom;


#[derive(Debug)]
pub enum FromDataError {
	ExpectedType(&'static str),
	Custom(&'static str),
	CustomString(String)
}


pub trait ColumnType: Sized {
	fn column_kind() -> ColumnKind;
	fn to_data(&self) -> ColumnData<'_>;
	fn from_data(data: ColumnData<'_>) -> Result<Self, FromDataError>;
}


macro_rules! imp {
	($type:ty, $kind:ident) => (
		imp!($type, $kind, |self| { *self }, |v| { Ok(v) });
	);
	(
		$type:ty, $kind:ident,
		|$self:ident| $block:block,
		|$v:ident| $from_block:block
	) => (
		impl ColumnType for $type {
			#[inline(always)]
			fn column_kind() -> ColumnKind {
				ColumnKind::$kind
			}
			fn to_data(&$self) -> ColumnData<'static> {
				ColumnData::$kind($block)
			}
			fn from_data(data: ColumnData) -> Result<Self, FromDataError> {
				match data {
					ColumnData::$kind($v) => {$from_block},
					_ => Err(FromDataError::ExpectedType(stringify!($kind)))
				}
			}
		}
	);
}


impl ColumnType for String {
	#[inline(always)]
	fn column_kind() -> ColumnKind {
		ColumnKind::Text
	}
	fn to_data(&self) -> ColumnData<'_> {
		ColumnData::Text(self.as_str().into())
	}
	fn from_data(data: ColumnData) -> Result<Self, FromDataError> {
		match data {
			ColumnData::Text(v) => Ok(v.into_string()),
			_ => Err(FromDataError::ExpectedType("Text"))
		}
	}
}

impl ColumnType for &str {
	#[inline(always)]
	fn column_kind() -> ColumnKind {
		ColumnKind::Text
	}
	fn to_data(&self) -> ColumnData<'_> {
		ColumnData::Text((*self).into())
	}
	fn from_data(data: ColumnData) -> Result<Self, FromDataError> {
		match data {
			ColumnData::Text(_) => {
				Err(FromDataError::Custom(
					"cannot convert string to &'static str"
				))
			},
			_ => Err(FromDataError::ExpectedType("Text"))
		}
	}
}
imp!(bool, Boolean);
/*imp!(char, Char(1),// maybe change this to char?
	|self| {*self as &str},
	|v| {v.chars().next().ok_or(FromDataError::Custom("no char found, expected 1"))}
);*/

imp!(f64, F64);
imp!(i64, I64);
imp!(u64, I64,
	|self| {i64::try_from(*self).expect("u64 to i64 overflowed")},
	|v| {Ok(v as u64)}
);// maybe should panic???

imp!(f32, F32);
imp!(i32, I32);
imp!(u32, I64,
	|self| {*self as i64},
	|v| {
		// maybe we dont need to convert
		u32::try_from(v)
			.map_err(|_| FromDataError::Custom("cannot convert i64 to u32"))
	}
);

imp!(i16, I16);
imp!(u16, I32,
	|self| {*self as i32},
	|v| {
		u16::try_from(v)
			.map_err(|_| FromDataError::Custom("cannot convert i32 to u32"))
	}
);

imp!(i8, I16,
	|self| {*self as i16},
	|v| {
		i8::try_from(v)
			.map_err(|_| FromDataError::Custom("cannot convert i16 to i8"))
	}
); // i thing there is another type??
imp!(u8, I16,
	|self| {*self as i16},
	|v| {
		u8::try_from(v)
			.map_err(|_| FromDataError::Custom("cannot convert i16 to u8"))
	}
); // maybe char


impl<T> ColumnType for Option<T>
where T: ColumnType {
	#[inline(always)]
	fn column_kind() -> ColumnKind {
		ColumnKind::Option(Box::new(T::column_kind()))
	}

	fn to_data(&self) -> ColumnData<'_> {
		ColumnData::Option(self.as_ref().map(|t| Box::new(t.to_data())))
	}

	fn from_data(data: ColumnData) -> Result<Self, FromDataError> {
		match data {
			ColumnData::Option(v) => match v {
				Some(v) => Ok(Some(T::from_data(*v)?)),
				None => Ok(None)
			},
			v => Ok(Some(T::from_data(v)?))
		}
	}
}

impl ColumnType for Vec<String> {
	#[inline(always)]
	fn column_kind() -> ColumnKind {
		ColumnKind::TextArray
	}

	fn to_data(&self) -> ColumnData<'_> {
		ColumnData::TextArray(self.as_slice().into())
	}

	fn from_data(data: ColumnData) -> Result<Self, FromDataError> {
		match data {
			ColumnData::TextArray(v) => Ok(v.into_vec_owned()),
			_ => Err(FromDataError::ExpectedType("expected TextArray"))
		}
	}
}

#[cfg(feature = "json")]
impl ColumnType for serde_json::Value {
	fn column_kind() -> ColumnKind {
		ColumnKind::Json
	}

	fn to_data(&self) -> ColumnData<'_> {
		let s = serde_json::to_string(self)
			.expect("could not serialize serde_json::Value");
		ColumnData::Text(s.into())
	}

	fn from_data(data: ColumnData) -> Result<Self, FromDataError> {
		match data {
			ColumnData::Text(s) => {
				serde_json::from_str(s.as_str())
					.map_err(|e| FromDataError::CustomString(e.to_string()))
			},
			_ => Err(FromDataError::ExpectedType("json string"))
		}
	}
}

#[cfg(feature = "email")]
impl ColumnType for email_address::EmailAddress {
	fn column_kind() -> ColumnKind {
		ColumnKind::Text
	}

	fn to_data(&self) -> ColumnData<'_> {
		ColumnData::Text(self.as_ref().into())
	}

	fn from_data(data: ColumnData) -> Result<Self, FromDataError> {
		match data {
			ColumnData::Text(s) => {
				s.as_str().parse()
					.map_err(|e: email_address::Error| {
						FromDataError::CustomString(e.to_string())
					})
			},
			_ => Err(FromDataError::ExpectedType("EmailAddress"))
		}
	}
}