
use std::time::Duration as StdDuration;
use std::fmt;

use serde::{Serialize, Deserialize};
use serde::ser::{Serializer, SerializeTuple};
use serde::de::{Deserializer, Visitor, SeqAccess, Error};



#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FullDuration(StdDuration);

impl FullDuration {
	pub fn new(secs: u64, nanos: u32) -> Self {
		Self(StdDuration::new(secs, nanos))
	}
}

impl Serialize for FullDuration {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where S: Serializer {
		let mut seq = serializer.serialize_tuple(2)?;
		seq.serialize_element(&self.0.as_secs())?;
		seq.serialize_element(&self.0.subsec_nanos())?;
		seq.end()
	}
}

pub struct FullDurationVisitor;
impl<'de> Visitor<'de> for FullDurationVisitor {
	type Value = FullDuration;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		write!(formatter, "a tuple of one u64 and one u32")
	}

	fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
	where A: SeqAccess<'de> {
		let secs = seq.next_element()?.ok_or(A::Error::missing_field("0: secs"))?;
		let nanos = seq.next_element()?.ok_or(A::Error::missing_field("1: nanos"))?;
		Ok(FullDuration::new(secs, nanos))
	}
}

impl<'de> Deserialize<'de> for FullDuration {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where D: Deserializer<'de> {
		deserializer.deserialize_tuple(2, FullDurationVisitor)
	}
}