use crate::table::column::{ColumnType, ColumnKind, ColumnData, FromDataError};
use std::time::{SystemTime, Duration};

use serde::{Serialize, Deserialize};
use serde::ser::{Serializer};
use serde::de::{Deserializer, Error};


#[derive(Debug, Clone)]
pub struct Timeout {
	inner: SystemTime
}

impl Timeout {
	pub fn new(dur: Duration) -> Self {
		Self {
			inner: SystemTime::now() + dur
		}
	}

	pub fn has_elapsed(&self) -> bool {
		SystemTime::now() > self.inner
	}

	/// returns the time from UNIX_EPOCH
	pub fn as_secs(&self) -> u64 {
		self.inner.duration_since(SystemTime::UNIX_EPOCH)
			.expect("Welcome to the past!")
			.as_secs()
	}

	pub fn from_secs(s: u64) -> Option<Self> {
		SystemTime::UNIX_EPOCH.checked_add(Duration::from_secs(s))
			.map(|c| Timeout { inner: c })
	}
}

impl Serialize for Timeout {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where S: Serializer {
		serializer.serialize_u64(self.as_secs())
	}
}

impl<'de> Deserialize<'de> for Timeout {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where D: Deserializer<'de> {
		let num: u64 = Deserialize::deserialize(deserializer)?;
		Self::from_secs(num)
			.ok_or(D::Error::custom("timeout to big"))
	}
}

impl ColumnType for Timeout {
	fn column_kind() -> ColumnKind {
		ColumnKind::I64
	}

	fn to_data(&self) -> ColumnData<'static> {
		ColumnData::I64(self.as_secs().try_into().expect("timeout to large"))
	}

	fn from_data(data: ColumnData) -> Result<Self, FromDataError> {
		match data {
			ColumnData::I64(u) => match u64::try_from(u) {
				Ok(u) => Self::from_secs(u)
					.ok_or(FromDataError::Custom("timeout to large")),
				Err(e) => Err(FromDataError::CustomString(e.to_string()))
			},
			_ => Err(FromDataError::ExpectedType("expected i64 for u64"))
		}
	}
}