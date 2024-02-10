use crate::table::{Table, TableTemplate};

use std::sync::Arc;

use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};

use tokio_postgres::Client;
use tokio_postgres::{connect, NoTls};

pub(crate) type SharedClient = Arc<RwLock<Client>>;

#[derive(Debug, Clone)]
pub struct Database {
	client: SharedClient,
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
		// let client = Client::with_uri_str("mongodb://localhost:27017")
		//	.expect("Failed to initilize mongo client.");
		let config = format!(
			"host={} dbname={} user={} password={}",
			host, name, user, password
		);

		let (client, connection) = connect(&config, NoTls)
			.await
			.expect("Failed to initialize postgres client");

		let client = Arc::new(RwLock::new(client));

		let bg_client = client.clone();
		tokio::spawn(async move {
			let mut connection = Some(connection);

			loop {
				if let Some(con) = connection.take() {
					if let Err(e) = con.await {
						tracing::error!("connection closed error: {}", e);
					}
				}

				sleep(Duration::from_secs(5)).await;

				let (client, con) = match connect(&config, NoTls).await {
					Ok(o) => o,
					Err(e) => {
						tracing::error!("connection error: {}", e);
						continue;
					}
				};

				// yeah, we got a new connection
				// let's replace the old one
				connection = Some(con);
				*bg_client.write().await = client;
			}
		});

		Self { client }
	}

	/// Get a table from the database
	pub fn table<T>(&self, name: &'static str) -> Table<T>
	where
		T: TableTemplate,
	{
		Table::new(self.client.clone(), name)
	}
}
