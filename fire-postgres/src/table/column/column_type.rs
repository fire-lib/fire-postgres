use super::ColumnKind;

pub trait ColumnType: Sized {
	fn column_kind() -> ColumnKind;
}

macro_rules! imp {
	($type:ty, $kind:ident) => {
		impl ColumnType for $type {
			#[inline(always)]
			fn column_kind() -> ColumnKind {
				ColumnKind::$kind
			}
		}
	};
}

imp!(String, Text);
imp!(&str, Text);

imp!(bool, Boolean);

imp!(f64, F64);
imp!(f32, F32);

imp!(i64, I64);
imp!(i32, I32);
imp!(i16, I16);
imp!(i8, I16);

impl<T> ColumnType for Option<T>
where
	T: ColumnType,
{
	#[inline(always)]
	fn column_kind() -> ColumnKind {
		ColumnKind::Option(Box::new(T::column_kind()))
	}
}

impl ColumnType for Vec<String> {
	#[inline(always)]
	fn column_kind() -> ColumnKind {
		ColumnKind::TextArray
	}
}

#[cfg(feature = "json")]
impl ColumnType for serde_json::Value {
	fn column_kind() -> ColumnKind {
		ColumnKind::Json
	}
}

#[cfg(feature = "email")]
impl ColumnType for email_address::EmailAddress {
	fn column_kind() -> ColumnKind {
		ColumnKind::Text
	}
}

#[cfg(feature = "json")]
mod impl_serde {
	use super::*;

	use postgres_types::Json;
	use serde::{de::DeserializeOwned, Serialize};

	impl<T> ColumnType for Json<T>
	where
		T: Serialize + DeserializeOwned,
	{
		fn column_kind() -> ColumnKind {
			ColumnKind::Json
		}
	}
}
