
// use serde::{Serialize, Deserialize};

mod datetime;
pub use datetime::DateTime;

/*mod duration;
pub use duration::Duration;

mod fullduration;
pub use fullduration::FullDuration;*/

// Maybe add full duration another time
// duration {secs: u64, nanos: u32}

/*#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Period {
	pub start: DateTime,
	pub duration: Duration
}*/