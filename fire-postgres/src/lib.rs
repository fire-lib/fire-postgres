#![allow(clippy::tabs_in_doc_comments)]
#![allow(clippy::never_loop)]
#![allow(clippy::new_without_default)]

#[cfg(feature = "connect")]
pub mod database;
#[cfg(feature = "connect")]
pub use database::Database;

pub mod table;
#[cfg(feature = "connect")]
pub use table::Table;

pub mod uid;
pub use uid::UniqueId;

pub mod time;

pub mod query;

pub use fire_postgres_derive::TableTempl;

#[cfg(feature = "connect")]
pub mod error;
#[cfg(feature = "connect")]
pub use error::Error;

#[cfg(feature = "connect")]
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(feature = "hash")]
pub mod hash;

mod macros;
pub mod utils;

mod impl_crypto;

// reexport for json macro
#[cfg(feature = "json")]
#[doc(hidden)]
pub use serde_json;
