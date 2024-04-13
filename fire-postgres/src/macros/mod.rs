/// non public facing utils for macros
#[doc(hidden)]
pub mod utils;

/// ## Example
/// ```
/// # use fire_postgres::try2;
///
/// fn mul(maybe_num: Option<i32>) -> Result<Option<i32>, &'static str> {
/// 	let x = try2!(maybe_num);
///
/// 	x.checked_mul(2)
/// 		.map(Some)
/// 		.ok_or("overflow")
/// }
/// ```
#[macro_export]
macro_rules! try2 {
	($exp:expr) => {
		match $exp {
			Some(o) => o,
			None => return Ok(None),
		}
	};
}

/// ## Example
/// ```
/// # use fire_postgres::try_vec;
///
/// fn add(maybe_vec: Option<Vec<i32>>) -> Result<Vec<i32>, &'static str> {
/// 	let v = try_vec!(maybe_vec);
///
/// 	Ok(v.into_iter().map(|x| x + 1).collect())
/// }
/// ```
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
macro_rules! enum_u16 {
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

			// fn to_data(&self) -> $crate::table::column::ColumnData<'_> {
			// 	$crate::table::column::ColumnData::I32(self.as_u16() as i32)
			// }

			// fn from_data(
			// 	data: $crate::table::column::ColumnData
			// ) -> std::result::Result<Self, $crate::table::column::FromDataError> {
			// 	use std::convert::TryFrom;
			// 	use $crate::table::column::FromDataError as __FromDataError;

			// 	match data {
			// 		$crate::table::column::ColumnData::I32(v) => {
			// 			let num = u16::try_from(v)
			// 				.map_err(|_| __FromDataError::Custom(
			// 					"cannot convert i32 to u32"
			// 				))?;
			// 			$name::from_u16(num)
			// 				.map_err(|m| __FromDataError::Custom(m))
			// 		},
			// 		_ => Err(__FromDataError::ExpectedType("u32"))
			// 	}
			// }
		}
	};
}
