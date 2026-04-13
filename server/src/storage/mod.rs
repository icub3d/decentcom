// The storage layer exposes a trait-based API that future features
// (auth, channels, messaging) will consume. Some items are defined ahead
// of their first caller, so scope dead-code lints to this module tree.
#![allow(dead_code)]

pub mod models;
pub mod sqlite;
pub mod traits;

use std::fmt;

pub use sqlite::SqliteStorage;
pub use traits::SessionStore;

#[derive(Debug)]
pub enum StorageError {
    NotFound,
    Conflict(String),
    Database(sqlx::Error),
    Migration(sqlx::migrate::MigrateError),
    Internal(String),
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::NotFound => write!(f, "not found"),
            StorageError::Conflict(msg) => write!(f, "conflict: {msg}"),
            StorageError::Database(e) => write!(f, "database error: {e}"),
            StorageError::Migration(e) => write!(f, "migration error: {e}"),
            StorageError::Internal(msg) => write!(f, "internal error: {msg}"),
        }
    }
}

impl std::error::Error for StorageError {}

impl From<sqlx::Error> for StorageError {
    fn from(e: sqlx::Error) -> Self {
        if let sqlx::Error::Database(ref db) = e {
            // SQLite UNIQUE constraint error code: 2067 (extended) / 19 (primary)
            if db.is_unique_violation() {
                return StorageError::Conflict(db.message().to_string());
            }
        }
        if matches!(e, sqlx::Error::RowNotFound) {
            return StorageError::NotFound;
        }
        StorageError::Database(e)
    }
}

impl From<sqlx::migrate::MigrateError> for StorageError {
    fn from(e: sqlx::migrate::MigrateError) -> Self {
        StorageError::Migration(e)
    }
}
