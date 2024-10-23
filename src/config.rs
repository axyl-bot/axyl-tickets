use sqlx::SqlitePool;
use std::env;
use std::sync::Arc;

pub struct Config {
    pub token: String,
    pub db: Arc<SqlitePool>,
}

impl Config {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut db_path = env::current_dir()?;
        db_path.push("axyl_tickets.db");
        let database_url = format!("sqlite:{}", db_path.display());
        let pool = SqlitePool::connect(&database_url).await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS config (
                key TEXT PRIMARY KEY,
                category_id INTEGER,
                log_channel_id INTEGER
            )",
        )
        .execute(&pool)
        .await?;

        sqlx::query("INSERT OR IGNORE INTO config (key) VALUES ('main')")
            .execute(&pool)
            .await?;

        let db = Arc::new(pool);

        Ok(Self {
            token: env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN must be set"),
            db,
        })
    }

    pub async fn get_category_id(&self) -> Result<Option<i64>, sqlx::Error> {
        sqlx::query_scalar!("SELECT category_id FROM config WHERE key = 'main'")
            .fetch_optional(&*self.db)
            .await
            .map(|opt| opt.flatten())
    }

    pub async fn set_category_id(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query!("UPDATE config SET category_id = ? WHERE key = 'main'", id)
            .execute(&*self.db)
            .await?;
        Ok(())
    }

    pub async fn get_log_channel_id(&self) -> Result<Option<i64>, sqlx::Error> {
        sqlx::query_scalar!("SELECT log_channel_id FROM config WHERE key = 'main'")
            .fetch_optional(&*self.db)
            .await
            .map(|opt| opt.flatten())
    }

    pub async fn set_log_channel_id(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE config SET log_channel_id = ? WHERE key = 'main'",
            id
        )
        .execute(&*self.db)
        .await?;
        Ok(())
    }
}
