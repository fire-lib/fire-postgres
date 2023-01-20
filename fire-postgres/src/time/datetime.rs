use crate::table::column::{ColumnType, ColumnKind, ColumnData, FromDataError};

use std::fmt;
use std::borrow::Cow;
use std::ops::{Add, Sub};
use std::time::{Duration as StdDuration, SystemTime};

use chrono::Utc;
use chrono::Duration;
use chrono::naive::NaiveDateTime;
use chrono::format::ParseError;
use chrono::offset::TimeZone;

use serde::{Serialize, Deserialize};
use serde::ser::Serializer;
use serde::de::{Deserializer, Error};


#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DateTime(chrono::DateTime<Utc>);

impl DateTime {
	pub fn new(secs: i64, ns: u32) -> Self {
		let naive = NaiveDateTime::from_timestamp_opt(secs, ns)
			.expect("secs and ns out of range");
		Self(chrono::DateTime::from_utc(naive, Utc))
	}

	pub fn now() -> Self {
		Self(Utc::now())
	}

	pub fn from_std(time: SystemTime) -> Self {
		Self(time.into())
	}

	pub fn from_secs(secs: i64) -> Self {
		Self::new(secs, 0)
	}

	// shouldnt this be i64
	pub fn from_ms(ms: u64) -> Self {
		let secs = ms / 1_000;
		let ns = (ms - (secs * 1_000)) * 1_000_000;

		Self::new(secs as i64, ns as u32)
	}

	pub fn raw(&self) -> &chrono::DateTime<Utc> {
		&self.0
	}

	pub fn raw_mut(&mut self) -> &mut chrono::DateTime<Utc> {
		&mut self.0
	}

	pub fn into_raw(self) -> chrono::DateTime<Utc> {
		self.0
	}

	pub fn to_microsecs_since_2000(&self) -> i64 {
		let date = Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap();
		self.0.clone().signed_duration_since(date).num_microseconds()
			.expect("value too large")
	}

	pub fn from_microsecs_since_2000(secs: i64) -> Self {
		let date = Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap();
		Self(date + Duration::microseconds(secs))
	}

	pub fn to_iso8601(&self) -> String {
		self.0.to_rfc3339()
	}

	pub fn parse_from_iso8601(s: &str) -> Result<Self, ParseError> {
		Ok(Self(s.parse()?))
	}

	/// Returns None if the duration would overflow
	pub fn abs_diff(&self, other: &Self) -> Option<StdDuration> {
		let mut diff = self.0 - other.0;
		if diff.num_seconds() < 0 {
			diff = diff * -1;
		}

		diff.to_std().ok()
	}
}

impl From<chrono::DateTime<Utc>> for DateTime {
	fn from(d: chrono::DateTime<Utc>) -> Self {
		Self(d)
	}
}

impl From<DateTime> for chrono::DateTime<Utc> {
	fn from(d: DateTime) -> Self {
		d.0
	}
}

impl Add<StdDuration> for DateTime {
	type Output = Self;

	/// ## Panic
	/// May panic if the duration is to big
	fn add(self, rhs: StdDuration) -> Self {
		Self(self.0 + Duration::from_std(rhs).unwrap())
	}
}

impl Sub<StdDuration> for DateTime {
	type Output = Self;

	fn sub(self, rhs: StdDuration) -> Self {
		Self(self.0 - Duration::from_std(rhs).unwrap())
	}
}

// TABLE INFO

impl ColumnType for DateTime {
	fn column_kind() -> ColumnKind {
		ColumnKind::Timestamp
	}
	fn to_data(&self) -> ColumnData<'_> {
		ColumnData::Timestamp(self.to_microsecs_since_2000())
	}
	fn from_data(data: ColumnData) -> Result<Self, FromDataError> {
		match data {
			ColumnData::Timestamp(m) => Ok(Self::from_microsecs_since_2000(m)),
			_ => Err(FromDataError::ExpectedType("Timestamp"))
		}
	}
}

// DISPLAY
impl fmt::Display for DateTime {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(&self.to_iso8601())
	}
}

// SERDE

impl Serialize for DateTime {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where S: Serializer {
		serializer.serialize_str(&self.to_iso8601())
	}
}

impl<'de> Deserialize<'de> for DateTime {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where D: Deserializer<'de> {
		let s: Cow<'_, str> = Deserialize::deserialize(deserializer)?;
		DateTime::parse_from_iso8601(s.as_ref())
			.map_err(D::Error::custom)
	}
}

#[cfg(test)]
mod tests {

	use super::*;
	use serde_json::{Value, from_value, from_str};

	#[test]
	fn serde_test() {
		let s = "\"2021-04-26T08:16:02+00:00\"";
		let d: DateTime = from_str(s).unwrap();
		assert_eq!(d.to_string(), "2021-04-26T08:16:02+00:00");

		let v = Value::String("2021-04-26T08:16:02+00:00".into());
		let d: DateTime = from_value(v).unwrap();
		assert_eq!(d.to_string(), "2021-04-26T08:16:02+00:00");
	}

}