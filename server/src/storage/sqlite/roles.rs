use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;

use super::SqliteStorage;
use crate::storage::models::{ChannelPermissionOverride, MemberRole, Role};
use crate::storage::traits::RoleStore;
use crate::storage::StorageError;

fn row_to_role(row: sqlx::sqlite::SqliteRow) -> Result<Role, StorageError> {
    Ok(Role {
        id: row.try_get("id")?,
        name: row.try_get("name")?,
        color: row.try_get("color")?,
        permissions: row.try_get("permissions")?,
        position: row.try_get("position")?,
        is_builtin: row.try_get::<i64, _>("is_builtin")? != 0,
        created_at: row.try_get::<DateTime<Utc>, _>("created_at")?,
        updated_at: row.try_get::<DateTime<Utc>, _>("updated_at")?,
    })
}

fn row_to_member_role(row: sqlx::sqlite::SqliteRow) -> Result<MemberRole, StorageError> {
    Ok(MemberRole {
        user_id: row.try_get("user_id")?,
        role_id: row.try_get("role_id")?,
    })
}

fn row_to_channel_override(
    row: sqlx::sqlite::SqliteRow,
) -> Result<ChannelPermissionOverride, StorageError> {
    Ok(ChannelPermissionOverride {
        channel_id: row.try_get("channel_id")?,
        role_id: row.try_get("role_id")?,
        allow: row.try_get("allow")?,
        deny: row.try_get("deny")?,
    })
}

#[async_trait]
impl RoleStore for SqliteStorage {
    async fn create_role(
        &self,
        name: &str,
        color: Option<&str>,
        permissions: i64,
        position: i32,
        is_builtin: bool,
    ) -> Result<Role, StorageError> {
        let id = self.new_id();
        sqlx::query(
            "INSERT INTO roles (id, name, color, permissions, position, is_builtin) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(name)
        .bind(color)
        .bind(permissions)
        .bind(position)
        .bind(if is_builtin { 1 } else { 0 })
        .execute(self.pool())
        .await?;

        self.get_role(&id).await?.ok_or(StorageError::NotFound)
    }

    async fn get_role(&self, id: &str) -> Result<Option<Role>, StorageError> {
        let row = sqlx::query("SELECT * FROM roles WHERE id = ?")
            .bind(id)
            .fetch_optional(self.pool())
            .await?;
        row.map(row_to_role).transpose()
    }

    async fn list_roles(&self) -> Result<Vec<Role>, StorageError> {
        let rows = sqlx::query("SELECT * FROM roles ORDER BY position DESC, created_at ASC")
            .fetch_all(self.pool())
            .await?;
        rows.into_iter().map(row_to_role).collect()
    }

    async fn update_role(
        &self,
        id: &str,
        name: Option<&str>,
        color: Option<Option<&str>>,
        permissions: Option<i64>,
        position: Option<i32>,
    ) -> Result<Role, StorageError> {
        match color {
            None => {
                sqlx::query(
                    "UPDATE roles
                     SET name = COALESCE(?, name),
                         permissions = COALESCE(?, permissions),
                         position = COALESCE(?, position),
                         updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
                     WHERE id = ?",
                )
                .bind(name)
                .bind(permissions)
                .bind(position)
                .bind(id)
                .execute(self.pool())
                .await?;
            }
            Some(color_value) => {
                sqlx::query(
                    "UPDATE roles
                     SET name = COALESCE(?, name),
                         color = ?,
                         permissions = COALESCE(?, permissions),
                         position = COALESCE(?, position),
                         updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
                     WHERE id = ?",
                )
                .bind(name)
                .bind(color_value)
                .bind(permissions)
                .bind(position)
                .bind(id)
                .execute(self.pool())
                .await?;
            }
        }

        self.get_role(id).await?.ok_or(StorageError::NotFound)
    }

    async fn delete_role(&self, id: &str) -> Result<(), StorageError> {
        let result = sqlx::query("DELETE FROM roles WHERE id = ?")
            .bind(id)
            .execute(self.pool())
            .await?;
        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound);
        }
        Ok(())
    }

    async fn add_member_role(
        &self,
        user_id: &str,
        role_id: &str,
    ) -> Result<MemberRole, StorageError> {
        sqlx::query("INSERT INTO member_roles (user_id, role_id) VALUES (?, ?)")
            .bind(user_id)
            .bind(role_id)
            .execute(self.pool())
            .await?;

        Ok(MemberRole {
            user_id: user_id.to_string(),
            role_id: role_id.to_string(),
        })
    }

    async fn remove_member_role(&self, user_id: &str, role_id: &str) -> Result<(), StorageError> {
        let result = sqlx::query("DELETE FROM member_roles WHERE user_id = ? AND role_id = ?")
            .bind(user_id)
            .bind(role_id)
            .execute(self.pool())
            .await?;
        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound);
        }
        Ok(())
    }

    async fn remove_all_member_roles(&self, user_id: &str) -> Result<(), StorageError> {
        sqlx::query("DELETE FROM member_roles WHERE user_id = ?")
            .bind(user_id)
            .execute(self.pool())
            .await?;
        Ok(())
    }

    async fn list_member_roles(&self, user_id: &str) -> Result<Vec<Role>, StorageError> {
        let rows = sqlx::query(
            "SELECT r.*
             FROM roles r
             INNER JOIN member_roles mr ON mr.role_id = r.id
             WHERE mr.user_id = ?
             ORDER BY r.position DESC, r.created_at ASC",
        )
        .bind(user_id)
        .fetch_all(self.pool())
        .await?;
        rows.into_iter().map(row_to_role).collect()
    }

    async fn list_role_members(&self, role_id: &str) -> Result<Vec<MemberRole>, StorageError> {
        let rows = sqlx::query(
            "SELECT user_id, role_id
             FROM member_roles
             WHERE role_id = ?
             ORDER BY user_id ASC",
        )
        .bind(role_id)
        .fetch_all(self.pool())
        .await?;
        rows.into_iter().map(row_to_member_role).collect()
    }

    async fn user_has_role(&self, user_id: &str, role_id: &str) -> Result<bool, StorageError> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*)
             FROM member_roles
             WHERE user_id = ? AND role_id = ?",
        )
        .bind(user_id)
        .bind(role_id)
        .fetch_one(self.pool())
        .await?;
        Ok(count > 0)
    }

    async fn upsert_channel_permission_override(
        &self,
        channel_id: &str,
        role_id: &str,
        allow: i64,
        deny: i64,
    ) -> Result<ChannelPermissionOverride, StorageError> {
        sqlx::query(
            "INSERT INTO channel_permission_overrides (channel_id, role_id, allow, deny)
             VALUES (?, ?, ?, ?)
             ON CONFLICT(channel_id, role_id) DO UPDATE SET
                 allow = excluded.allow,
                 deny = excluded.deny",
        )
        .bind(channel_id)
        .bind(role_id)
        .bind(allow)
        .bind(deny)
        .execute(self.pool())
        .await?;

        let row = sqlx::query(
            "SELECT channel_id, role_id, allow, deny
             FROM channel_permission_overrides
             WHERE channel_id = ? AND role_id = ?",
        )
        .bind(channel_id)
        .bind(role_id)
        .fetch_optional(self.pool())
        .await?;

        row.map(row_to_channel_override)
            .transpose()?
            .ok_or(StorageError::NotFound)
    }

    async fn delete_channel_permission_override(
        &self,
        channel_id: &str,
        role_id: &str,
    ) -> Result<(), StorageError> {
        let result = sqlx::query(
            "DELETE FROM channel_permission_overrides WHERE channel_id = ? AND role_id = ?",
        )
        .bind(channel_id)
        .bind(role_id)
        .execute(self.pool())
        .await?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound);
        }

        Ok(())
    }

    async fn list_channel_permission_overrides(
        &self,
        channel_id: &str,
    ) -> Result<Vec<ChannelPermissionOverride>, StorageError> {
        let rows = sqlx::query(
            "SELECT channel_id, role_id, allow, deny
             FROM channel_permission_overrides
             WHERE channel_id = ?
             ORDER BY role_id ASC",
        )
        .bind(channel_id)
        .fetch_all(self.pool())
        .await?;

        rows.into_iter().map(row_to_channel_override).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::traits::{ChannelStore, UserStore};

    #[tokio::test]
    async fn role_and_override_crud() {
        let s = SqliteStorage::in_memory().await.unwrap();
        let role = s
            .create_role("mods", Some("#ff00ff"), 3, 10, false)
            .await
            .unwrap();

        let fetched = s.get_role(&role.id).await.unwrap().unwrap();
        assert_eq!(fetched.name, "mods");

        let updated = s
            .update_role(&role.id, Some("mods2"), Some(None), Some(7), Some(20))
            .await
            .unwrap();
        assert_eq!(updated.name, "mods2");
        assert!(updated.color.is_none());

        let channel = s.create_channel("general", None, 0).await.unwrap();
        let override_row = s
            .upsert_channel_permission_override(&channel.id, &role.id, 1, 2)
            .await
            .unwrap();
        assert_eq!(override_row.allow, 1);
        assert_eq!(override_row.deny, 2);

        let listed = s
            .list_channel_permission_overrides(&channel.id)
            .await
            .unwrap();
        assert_eq!(listed.len(), 1);

        s.delete_channel_permission_override(&channel.id, &role.id)
            .await
            .unwrap();
        let listed_after = s
            .list_channel_permission_overrides(&channel.id)
            .await
            .unwrap();
        assert!(listed_after.is_empty());
    }

    #[tokio::test]
    async fn member_role_assignment() {
        let s = SqliteStorage::in_memory().await.unwrap();
        let user = s.create_user("pk-role", None, false).await.unwrap();

        let roles = s.list_roles().await.unwrap();
        let everyone = roles.into_iter().find(|r| r.id == "everyone").unwrap();

        s.add_member_role(&user.id, &everyone.id).await.unwrap();
        assert!(s.user_has_role(&user.id, &everyone.id).await.unwrap());

        let member_roles = s.list_member_roles(&user.id).await.unwrap();
        assert_eq!(member_roles.len(), 1);

        s.remove_member_role(&user.id, &everyone.id).await.unwrap();
        assert!(!s.user_has_role(&user.id, &everyone.id).await.unwrap());
    }
}
