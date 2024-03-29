#[cfg(feature = "crypto-cipher")]
mod cipher {

	use crate::table::column::{
		ColumnData, ColumnKind, ColumnType, FromDataError,
	};
	use crypto::cipher::{Keypair, PublicKey};
	use std::str::FromStr;

	impl ColumnType for Keypair {
		fn column_kind() -> ColumnKind {
			ColumnKind::FixedText(43)
		}

		fn to_data(&self) -> ColumnData<'_> {
			ColumnData::Text(self.to_string().into())
		}

		fn from_data(data: ColumnData<'_>) -> Result<Self, FromDataError> {
			match data {
				ColumnData::Text(t) => {
					Self::from_str(t.as_str()).map_err(|_| {
						FromDataError::Custom(
							"could not derive keypair from string",
						)
					})
				}
				_ => Err(FromDataError::ExpectedType("Keypair text")),
			}
		}
	}

	impl ColumnType for PublicKey {
		fn column_kind() -> ColumnKind {
			ColumnKind::FixedText(43)
		}

		fn to_data(&self) -> ColumnData<'_> {
			ColumnData::Text(self.to_string().into())
		}

		fn from_data(data: ColumnData<'_>) -> Result<Self, FromDataError> {
			match data {
				ColumnData::Text(t) => {
					Self::from_str(t.as_str()).map_err(|_| {
						FromDataError::Custom(
							"could not derive publickey from string",
						)
					})
				}
				_ => Err(FromDataError::ExpectedType("PublicKey text")),
			}
		}
	}
}

#[cfg(feature = "crypto-signature")]
mod signature {

	use crate::table::column::{
		ColumnData, ColumnKind, ColumnType, FromDataError,
	};
	use crypto::signature::{Keypair, PublicKey, Signature};
	use std::str::FromStr;

	impl ColumnType for Keypair {
		fn column_kind() -> ColumnKind {
			ColumnKind::FixedText(43)
		}

		fn to_data(&self) -> ColumnData<'_> {
			ColumnData::Text(self.to_string().into())
		}

		fn from_data(data: ColumnData<'_>) -> Result<Self, FromDataError> {
			match data {
				ColumnData::Text(t) => {
					Self::from_str(t.as_str()).map_err(|_| {
						FromDataError::Custom(
							"could not derive keypair from string",
						)
					})
				}
				_ => Err(FromDataError::ExpectedType("Keypair text")),
			}
		}
	}

	impl ColumnType for PublicKey {
		fn column_kind() -> ColumnKind {
			ColumnKind::FixedText(43)
		}

		fn to_data(&self) -> ColumnData<'_> {
			ColumnData::Text(self.to_string().into())
		}

		fn from_data(data: ColumnData<'_>) -> Result<Self, FromDataError> {
			match data {
				ColumnData::Text(t) => {
					Self::from_str(t.as_str()).map_err(|_| {
						FromDataError::Custom(
							"could not derive publickey from string",
						)
					})
				}
				_ => Err(FromDataError::ExpectedType("PublicKey text")),
			}
		}
	}

	impl ColumnType for Signature {
		fn column_kind() -> ColumnKind {
			ColumnKind::FixedText(86)
		}

		fn to_data(&self) -> ColumnData<'_> {
			ColumnData::Text(self.to_string().into())
		}

		fn from_data(data: ColumnData<'_>) -> Result<Self, FromDataError> {
			match data {
				ColumnData::Text(t) => {
					Self::from_str(t.as_str()).map_err(|_| {
						FromDataError::Custom(
							"could not derive publickey from string",
						)
					})
				}
				_ => Err(FromDataError::ExpectedType("PublicKey text")),
			}
		}
	}
}

#[cfg(feature = "crypto-token")]
mod token {

	use crate::table::column::{
		ColumnData, ColumnKind, ColumnType, FromDataError,
	};
	use crypto::token::Token;
	use std::str::FromStr;

	impl<const S: usize> ColumnType for Token<S> {
		fn column_kind() -> ColumnKind {
			ColumnKind::FixedText(Self::STR_LEN)
		}

		fn to_data(&self) -> ColumnData<'_> {
			ColumnData::Text(self.to_string().into())
		}

		fn from_data(data: ColumnData<'_>) -> Result<Self, FromDataError> {
			match data {
				ColumnData::Text(t) => {
					Self::from_str(t.as_str()).map_err(|_| {
						FromDataError::Custom(
							"could not derive keypair from string",
						)
					})
				}
				_ => Err(FromDataError::ExpectedType("Keypair text")),
			}
		}
	}
}
