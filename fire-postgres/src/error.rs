use crate::table::column::FromDataError;
use crate::table::info::ValidateParamsError;

use std::fmt;

use tokio_postgres::error::Error as PostgresError;

#[cfg(feature = "hash")]
use bcrypt::BcryptError;

#[derive(Debug)]
pub enum Error {
	Postgres(String),
	FromData(FromDataError),
	ValidateParamsError(ValidateParamsError),
	#[cfg(feature = "hash")]
	BcryptError(BcryptError),
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		fmt::Debug::fmt(self, f)
	}
}

impl std::error::Error for Error {}

impl From<PostgresError> for Error {
	fn from(e: PostgresError) -> Self {
		Self::Postgres(format!("{:?}", e))
	}
}

impl From<FromDataError> for Error {
	fn from(e: FromDataError) -> Self {
		Self::FromData(e)
	}
}

impl From<ValidateParamsError> for Error {
	fn from(e: ValidateParamsError) -> Self {
		Self::ValidateParamsError(e)
	}
}

#[cfg(feature = "hash")]
impl From<BcryptError> for Error {
	fn from(e: BcryptError) -> Self {
		Self::BcryptError(e)
	}
}
