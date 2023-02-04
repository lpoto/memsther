use std::ops::DerefMut;

use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod};
use tokio_postgres::NoTls;

pub mod user;

mod embedded {
    refinery::embed_migrations!("migrations");
}

#[derive(serde::Deserialize)]
pub struct Configuration {
    host: String,
    user: String,
    password: String,
    dbname: String,
}

pub struct Datastore {
    pub pool: Pool,
}

impl Datastore {
    /// Create a new Datastore object that handles
    /// the postgresql databse pool and panic
    /// when the pool could not be created.
    pub fn new(config: Configuration) -> Datastore {
        let mut dp_config = Config::new();
        let dbname = config.dbname.clone();
        dp_config.host = Some(config.host);
        dp_config.user = Some(config.user);
        dp_config.password = Some(config.password);
        dp_config.dbname = Some(config.dbname);

        dp_config.manager = Some(ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
            ..Default::default()
        });
        let pool = match dp_config.create_pool(NoTls) {
            | Ok(pool) => pool,
            | Err(e) => panic!("Could not create a postgres pool: {}", e),
        };
        log::info!("Created pool for postgresql database: {}", dbname);
        Datastore {
            pool,
        }
    }

    /// Run migrations located in ../migrations/ with refinery.
    /// This may panic either due to invalid postgres connection
    /// or failure to apply any of the migrations.
    pub async fn migrate(&self) {
        log::debug!("Running postgres migrations ...");
        let mut client = match self.pool.get().await {
            | Ok(client) => client,
            | Err(err) => panic!("Err {:?}", err),
        };
        let client = client.deref_mut().deref_mut();
        match embedded::migrations::runner().run_async(client).await {
            | Ok(report) => log::info!(
                "Applied {} migration/s",
                report.applied_migrations().len()
            ),
            | Err(err) => panic!("Error running migrations: {}", err),
        };
    }
}
