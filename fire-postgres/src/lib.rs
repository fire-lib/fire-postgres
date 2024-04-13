#![allow(clippy::tabs_in_doc_comments)]
#![allow(clippy::never_loop)]
#![allow(clippy::new_without_default)]

pub mod database;
pub use database::Database;

pub mod connection;
pub use connection::Connection;
pub use connection::Error;

pub mod row;
pub use row::Row;

pub mod table;
// pub use table::Table;

pub use types::time;
pub use types::uid::UniqueId;

pub mod filter;

pub mod update;

pub use fire_postgres_derive::{FromRow, TableTempl};

pub type Result<T> = std::result::Result<T, Error>;

mod macros;

// mod impl_crypto;
