
use std::time::Duration as StdDuration;

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Duration(u64); // secs

impl Duration {

	pub fn new(secs: u64) -> Self {
		Self(secs)
	}

	pub fn as_secs(&self) -> u64 {
		self.0
	}

	pub fn into_std(self) -> StdDuration {
		StdDuration::from_secs(self.0)
	}

}

impl From<StdDuration> for Duration {
	fn from(d: StdDuration) -> Self {
		Self::new(d.as_secs())
	}
}

impl Into<StdDuration> for Duration {
	fn into(self) -> StdDuration {
		self.into_std()
	}
}