use crate::table::{Table, TableTemplate};

use deadpool_postgres::{Pool, Runtime};

use tokio_postgres::NoTls;

#[derive(Debug, Clone)]
pub struct Database {
	pool: Pool,
}

impl Database {
	/// Create a new database
	pub async fn new(name: &str, user: &str, password: &str) -> Self {
		Self::with_host("localhost", name, user, password).await
	}

	/// Create a new database with a host
	pub async fn with_host(
		host: &str,
		name: &str,
		user: &str,
		password: &str,
	) -> Self {
		let config = deadpool_postgres::Config {
			host: Some(host.to_string()),
			dbname: Some(name.to_string()),
			user: Some(user.to_string()),
			password: Some(password.to_string()),
			..Default::default()
		};

		let pool = config.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();

		// let's see if we can get a connection
		let _client =
			pool.get().await.expect("could not get a postgres client");

		Self { pool }
	}

	/// Get a table from the database
	pub fn table<T>(&self, name: &'static str) -> Table<T>
	where
		T: TableTemplate,
	{
		Table::new(self.pool.clone(), name)
	}
}
