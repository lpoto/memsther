use std::{env, ops::DerefMut};

use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod};
use tokio_postgres::NoTls;
use url::Url;

pub mod user;

mod embedded {
    refinery::embed_migrations!("migrations");
}

pub struct Datastore {
    pub pool: Pool,
}

impl Datastore {
    /// Create a new Datastore object that handles
    /// the postgresql databse pool and panic
    /// when the pool could not be created.
    pub fn new() -> Datastore {
        let mut dp_config = Config::new();
        let database_url = env::var("DATABASE_URL")
            .expect("missing DATABASE_URL env variable");
        let url = Url::parse(database_url.as_str())
            .expect("invalid DATABASE_URL env variable");
        let user = url.username().to_string();
        let password = url
            .password()
            .ok_or("")
            .expect("invalid password in DATABASE_URL")
            .to_string();
        let host = url
            .host()
            .ok_or("")
            .expect("invalid host in DATABASE_URL")
            .to_string();
        let port = url.port().ok_or("").expect("invalid port in DATABASE_URL");
        let dbname = url
            .path_segments()
            .expect("invalid database name in DATABASE_URL")
            .next()
            .unwrap()
            .to_string();

        dp_config.host = Some(host);
        dp_config.user = Some(user);
        dp_config.port = Some(port);
        dp_config.password = Some(password);
        dp_config.dbname = Some(dbname.clone());
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
