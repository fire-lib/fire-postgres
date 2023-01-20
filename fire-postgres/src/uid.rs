
use crate::table::column::{ColumnType, ColumnKind, ColumnData, FromDataError};

use std::time::{SystemTime, UNIX_EPOCH};
use std::fmt;
use std::str::FromStr;
use std::borrow::Cow;

use rand::{RngCore, rngs::OsRng};
use base64::{encode_config, decode_config_slice, URL_SAFE_NO_PAD, DecodeError};

use serde::{Serialize, Deserialize};
use serde::ser::Serializer;
use serde::de::{Deserializer, Error};

/// A UniqueId that can be used within a database. 
/// Is not cryptographically secure and could be bruteforced.
///
/// Contains 10bytes
/// - 0..5 are seconds since the UNIX_EPOCH 
/// - 5..10 are random
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct UniqueId([u8; 10]);

impl UniqueId {

	pub fn new() -> Self {
		let secs_bytes = SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.expect("SystemTime before UNIX EPOCH!")
			.as_secs()
			.to_be_bytes();

		let mut bytes = [0u8; 10];
		bytes[..5].copy_from_slice(&secs_bytes[3..8]);

		OsRng.fill_bytes(&mut bytes[5..]);

		Self(bytes)
	}

	/// This creates a unique id with it's raw content
	/// making it able to be called in a const context.
	pub const fn from_raw(inner: [u8; 10]) -> Self {
		Self(inner)
	}

	pub fn from_slice_unchecked(slice: &[u8]) -> Self {
		let mut bytes = [0u8; 10];
		bytes.copy_from_slice(slice);
		Self(bytes)
	}

	pub fn to_b64(&self) -> String {
		encode_config(&self.0, URL_SAFE_NO_PAD)
	}

	// this panics if b64 has not a length of 14
	pub fn parse_from_b64<T>(b64: T) -> Result<Self, DecodeError>
	where T: AsRef<[u8]> {
		let mut bytes = [0u8; 10];
		decode_config_slice(b64, URL_SAFE_NO_PAD, &mut bytes)?;
		Ok(Self(bytes))
	}

	pub fn from_bytes(bytes: [u8; 10]) -> Self {
		Self(bytes)
	}

	pub fn into_bytes(self) -> [u8; 10] {
		self.0
	}

	pub fn since_unix_secs(&self) -> u64 {
		let mut bytes = [0u8; 8];
		bytes[3..].copy_from_slice(&self.0[..5]);
		u64::from_be_bytes(bytes)
	}

	pub fn as_slice(&self) -> &[u8] {
		&self.0
	}

}

impl fmt::Debug for UniqueId {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_tuple("UniqueId")
			.field(&self.to_b64())
			.finish()
	}
}

impl fmt::Display for UniqueId {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.to_b64().fmt(f)
	}
}

impl FromStr for UniqueId {
	type Err = DecodeError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Self::parse_from_b64(s)
	}
}

impl From<DecodeError> for FromDataError {
	fn from(e: DecodeError) -> Self {
		Self::CustomString(format!("uniqueid decode error {:?}", e))
	}
}

/*
faster insert even after b64 overhead
insert_time: 884306ns
select_time: 122143ns
*/
#[cfg(not(feature = "uid-as-bytea"))]
impl ColumnType for UniqueId {

	fn column_kind() -> ColumnKind {
		ColumnKind::FixedText(14)
	}

	fn to_data(&self) -> ColumnData<'_> {
		ColumnData::Text(self.to_b64().into())
	}

	fn from_data(data: ColumnData) -> Result<Self, FromDataError> {
		match data {
			ColumnData::Text(s) if s.len() == 14 => Ok(Self::parse_from_b64(s.as_str())?),
			_ => Err(FromDataError::ExpectedType("char with 14 chars for unique id"))
		}
	}

}

/*
slower on insert but database must be slower (there is no b64 overhead)
insert_time: 909610ns
select_time: 117526ns
*/
#[cfg(feature = "uid-as-bytea")]
impl ColumnType for UniqueId {

	fn column_kind() -> ColumnKind {
		ColumnKind::Bytea
	}

	fn to_data(&self) -> ColumnData<'_> {
		ColumnData::Bytea(self.as_slice())
	}

	fn from_data(data: ColumnData) -> Result<Self, FromDataError> {
		match data {
			ColumnData::Bytea(s) if s.len() == 10 => Ok(Self::from_slice_unchecked(s)),
			_ => Err(FromDataError::ExpectedType("bytea with 10 bytes for unique id"))
		}
	}

}

// SERDE

impl Serialize for UniqueId {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where S: Serializer {
		serializer.serialize_str(&self.to_b64())
	}
}

impl<'de> Deserialize<'de> for UniqueId {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where D: Deserializer<'de> {
		let s: Cow<'_, str> = Deserialize::deserialize(deserializer)?;
		let s = s.as_ref();
		if s.len() == 14 {
			UniqueId::parse_from_b64(s)
				.map_err(D::Error::custom)
		} else {
			Err(D::Error::custom("expected string with exactly 14 characters"))
		}
	}
}


#[cfg(test)]
mod tests {

	use super::*;
	use serde_json::{Value, from_value, from_str};

	#[test]
	fn serde_test() {
		let s = "\"AGCGeWIDTlipbg\"";
		let d: UniqueId = from_str(s).unwrap();
		assert_eq!(d.to_string(), "AGCGeWIDTlipbg");

		let v = Value::String("AGCGeWIDTlipbg".into());
		let d: UniqueId = from_value(v).unwrap();
		assert_eq!(d.to_string(), "AGCGeWIDTlipbg");
	}

}