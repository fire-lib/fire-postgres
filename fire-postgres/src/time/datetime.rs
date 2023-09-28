use super::Date;
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

/// A DateTime in the utc timezone
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
		self.0.signed_duration_since(date).num_microseconds()
			.expect("value too large")
	}

	pub fn from_microsecs_since_2000(secs: i64) -> Self {
		let date = Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap();
		Self(date + Duration::microseconds(secs))
	}

	pub fn to_iso8601(&self) -> String {
		self.0.to_rfc3339()
	}

	pub fn to_date(&self) -> Date {
		self.0.date_naive().into()
	}

	pub fn parse_from_iso8601(s: &str) -> Result<Self, ParseError> {
		Ok(Self(s.parse()?))
	}

	/// Returns None if the duration would overflow
	pub fn abs_diff(&self, other: &Self) -> Option<StdDuration> {
		(self.0 - other.0).abs().to_std().ok()
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


#[cfg(feature = "protobuf")]
mod protobuf {
	use super::*;

	use fire_protobuf::{
		WireType,
		encode::{
			EncodeMessage, MessageEncoder, FieldOpt, SizeBuilder, EncodeError
		},
		decode::{DecodeMessage, FieldKind, DecodeError},
		bytes::BytesWrite
	};

	impl EncodeMessage for DateTime {
		const WIRE_TYPE: WireType = WireType::Varint;

		fn is_default(&self) -> bool {
			false
		}

		fn encoded_size(
			&mut self,
			field: Option<FieldOpt>,
			builder: &mut SizeBuilder
		) -> Result<(), EncodeError> {
			self.to_microsecs_since_2000().encoded_size(field, builder)
		}

		fn encode<B>(
			&mut self,
			field: Option<FieldOpt>,
			encoder: &mut MessageEncoder<B>
		) -> Result<(), EncodeError>
		where B: BytesWrite {
			self.to_microsecs_since_2000().encode(field, encoder)
		}
	}

	impl<'m> DecodeMessage<'m> for DateTime {
		const WIRE_TYPE: WireType = WireType::Varint;

		fn decode_default() -> Self {
			Self::from_microsecs_since_2000(0)
		}

		fn merge(
			&mut self,
			kind: FieldKind<'m>,
			is_field: bool
		) -> Result<(), DecodeError> {
			let mut n = 0i64;
			n.merge(kind, is_field)?;

			*self = Self::from_microsecs_since_2000(n);

			Ok(())
		}
	}
}

#[cfg(feature = "graphql")]
mod graphql {
	use super::*;

	use juniper::{graphql_scalar, Value};

	#[graphql_scalar]
	impl<S> GraphQlScalar for DateTime
	where S: ScalarValue {
		fn resolve(&self) -> Value {
			Value::scalar(self.to_string())
		}

		fn from_input_value(value: &InputValue) -> Option<DateTime> {
			value.as_string_value()
				.and_then(|s| DateTime::parse_from_iso8601(s.as_ref()).ok())
		}

		fn from_str<'a>(
			value: ScalarToken<'a>
		) -> juniper::ParseScalarResult<'a, S> {
			<String as juniper::ParseScalarValue<S>>::from_str(value)
		}
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