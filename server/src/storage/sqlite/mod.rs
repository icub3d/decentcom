mod channels;
mod messages;
mod sessions;
mod users;

use std::path::Path;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use sqlx::migrate::MigrateError;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::SqlitePool;
use tracing::warn;
use ulid::Generator;

use super::StorageError;

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

pub struct SqliteStorage {
    pool: SqlitePool,
    id_gen: Mutex<Generator>,
}

impl SqliteStorage {
    /// Open (and create, if missing) a SQLite database at the given filesystem path.
    pub async fn open(path: &Path) -> Result<Self, StorageError> {
        let opts = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .connect_with(opts)
            .await
            .map_err(StorageError::from)?;
        if let Err(err) = MIGRATOR.run(&pool).await {
            if cfg!(debug_assertions) && matches!(err, MigrateError::VersionMismatch(_)) {
                drop(pool);

                let backup_path = backup_path_for(path);
                std::fs::rename(path, &backup_path).map_err(|e| {
                    StorageError::Internal(format!(
                        "failed to backup sqlite database after migration mismatch: {e}"
                    ))
                })?;

                warn!(
                    db_path = %path.display(),
                    backup_path = %backup_path.display(),
                    "migration version mismatch detected; backed up old database and recreating a fresh one"
                );

                let recreated_opts = SqliteConnectOptions::new()
                    .filename(path)
                    .create_if_missing(true)
                    .journal_mode(SqliteJournalMode::Wal)
                    .foreign_keys(true);

                let recreated_pool = SqlitePoolOptions::new()
                    .connect_with(recreated_opts)
                    .await
                    .map_err(StorageError::from)?;

                MIGRATOR.run(&recreated_pool).await?;

                return Ok(Self {
                    pool: recreated_pool,
                    id_gen: Mutex::new(Generator::new()),
                });
            }

            return Err(StorageError::from(err));
        }

        Ok(Self {
            pool,
            id_gen: Mutex::new(Generator::new()),
        })
    }

    /// Open an in-memory database, used for tests.
    pub async fn in_memory() -> Result<Self, StorageError> {
        let opts = SqliteConnectOptions::new()
            .filename(":memory:")
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .foreign_keys(true);
        // Each `:memory:` connection is an isolated database, so the pool
        // must be limited to one connection for migrations and queries to
        // share state.
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(opts)
            .await
            .map_err(StorageError::from)?;
        MIGRATOR.run(&pool).await?;
        Ok(Self {
            pool,
            id_gen: Mutex::new(Generator::new()),
        })
    }

    pub(crate) fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub(crate) fn new_id(&self) -> String {
        let mut gen = self.id_gen.lock().expect("id generator mutex poisoned");
        gen.generate()
            .expect("ulid generator overflow within single millisecond")
            .to_string()
    }
}

fn backup_path_for(path: &Path) -> std::path::PathBuf {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("decentcom.db");

    path.with_file_name(format!("{file_name}.bak-{ts}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn initializes_and_runs_migrations() {
        let s = SqliteStorage::in_memory().await.unwrap();
        // Verify the migrations created the expected tables.
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM sqlite_master WHERE type='table'")
                .fetch_one(s.pool())
                .await
                .unwrap();
        assert!(count >= 6, "expected at least 6 tables, got {count}");
    }

    #[tokio::test]
    async fn ids_are_unique_and_monotonic() {
        let s = SqliteStorage::in_memory().await.unwrap();
        let a = s.new_id();
        let b = s.new_id();
        assert_ne!(a, b);
        assert!(b > a);
    }
}
