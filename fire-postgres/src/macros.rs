#[macro_export]
macro_rules! try_res_opt {
	($exp:expr) => {
		match $exp {
			Ok(o) => $crate::try2!(o),
			Err(e) => return Err(e.into()),
		}
	};
}

#[macro_export]
macro_rules! try2 {
	($exp:expr) => {
		match $exp {
			Some(o) => o,
			None => return Ok(None),
		}
	};
}

#[macro_export]
macro_rules! try_vec {
	($exp:expr) => {
		match $exp {
			Some(o) => o,
			None => return Ok(vec![]),
		}
	};
}

/// ## Example
/// ```
/// use fire_postgres::enum_u16;
/// enum_u16! {
/// 	#[derive(Debug)]
/// 	pub enum SiteRaw {
/// 		FrameUser = 10,
/// 		Admin = 20,
/// 		App = 30
/// 	}
/// }
/// ```
#[macro_export]
macro_rules! enum_u16 {// u32
	($(#[$metas:meta])* $($pub:ident)? enum $name:ident {
		$($opt:ident = $num:expr),*
	}) => {
		$(#[$metas])*
		#[repr(u16)]
		$($pub)? enum $name {
			$($opt = $num),*
		}

		impl $name {

			pub fn as_u16(&self) -> u16 {
				match self {
					$(Self::$opt => $num),*
				}
			}

			pub fn from_u16(
				num: u16
			) -> std::result::Result<Self, &'static str> {
				match num {
					$($num => Ok(Self::$opt)),*,
					_ => Err(stringify!(could not parse u16 to $name))
				}
			}

		}

		impl $crate::table::column::ColumnType for $name {

			fn column_kind() -> $crate::table::column::ColumnKind {
				$crate::table::column::ColumnKind::I32
			}

			fn to_data(&self) -> $crate::table::column::ColumnData<'_> {
				$crate::table::column::ColumnData::I32(self.as_u16() as i32)
			}

			fn from_data(
				data: $crate::table::column::ColumnData
			) -> std::result::Result<Self, $crate::table::column::FromDataError> {
				use std::convert::TryFrom;
				use $crate::table::column::FromDataError as __FromDataError;

				match data {
					$crate::table::column::ColumnData::I32(v) => {
						let num = u16::try_from(v)
							.map_err(|_| __FromDataError::Custom(
								"cannot convert i32 to u32"
							))?;
						$name::from_u16(num)
							.map_err(|m| __FromDataError::Custom(m))
					},
					_ => Err(__FromDataError::ExpectedType("u32"))
				}
			}

		}
	};
}

/// ## Note
/// This panics if the data cannot be serialized.
#[cfg(feature = "json")]
#[macro_export]
macro_rules! impl_json_col_type {
	($struct:ident) => {
		impl $crate::table::column::ColumnType for $struct {
			fn column_kind() -> $crate::table::column::ColumnKind {
				$crate::table::column::ColumnKind::Json
			}

			fn to_data(&self) -> $crate::table::column::ColumnData<'_> {
				let s = $crate::serde_json::to_string(self).expect(&format!(
					"could not serialize {}",
					stringify!($struct)
				));
				$crate::table::column::ColumnData::Text(s.into())
			}

			fn from_data(
				data: $crate::table::column::ColumnData,
			) -> std::result::Result<Self, $crate::table::column::FromDataError>
			{
				match data {
					$crate::table::column::ColumnData::Text(s) => {
						$crate::serde_json::from_str(s.as_str()).map_err(|e| {
							$crate::table::column::FromDataError::CustomString(
								e.to_string(),
							)
						})
					}
					_ => {
						Err($crate::table::column::FromDataError::ExpectedType(
							"str for json",
						))
					}
				}
			}
		}
	};
}
