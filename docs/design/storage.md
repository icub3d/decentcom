# Storage Backends

## Design Goals

The storage layer should be:
- **Pluggable** — operators choose a backend based on their scale and operational preference.
- **Abstracted** — the server core talks to a storage interface; backend details are hidden behind it.
- **Sane by default** — SQLite + local disk should work out of the box with zero configuration.
- **Scalable when needed** — a large hosted deployment should be able to use PostgreSQL + object storage without changing any application logic.

## What Gets Stored

| Data Type | Characteristics |
|---|---|
| User records | Small, write-once (pubkey, display name, avatar hash) |
| Messages | Append-heavy, high read volume, never truly deleted (soft-deleted) |
| Channel / role / server config | Small, infrequently written |
| Media (images, files, video) | Large blobs, high read volume, write-once |
| Voice / video streams | Not persisted (unless recording enabled) |
| Audit logs | Append-only |
| Invite records | Small, TTL-based expiry |
| Session tokens | Short-lived, high churn |

## Storage Interface

The application defines a storage trait (Rust `trait`) that covers all persistence operations. Backends implement this trait. This keeps business logic independent of storage technology.

```
StorageBackend
  ├── UserStore
  ├── MessageStore
  ├── ChannelStore
  ├── MediaStore
  └── SessionStore
```

Session tokens are likely best kept in a fast in-memory store (or an in-process cache) rather than the primary database. If the server restarts, sessions can be reissued via re-authentication.

## Backends

### SQLite + Local Disk (Default)

**Use case:** Home servers, small communities, single-node deployments.

- All relational data (users, messages, channels, roles) stored in a single SQLite file.
- Media files stored in a local directory, referenced by content hash (content-addressable storage).
- Zero external dependencies. Works offline.
- sqlx with SQLite driver in Rust.

**Limits:**
- Write concurrency is limited by SQLite's WAL mode. This is sufficient for hundreds of concurrent users but not thousands.
- Not suitable for multi-node deployments.

**Configuration example:**
```toml
[storage]
backend = "sqlite"
database_path = "/var/lib/decentcom/db.sqlite"
media_path = "/var/lib/decentcom/media"
```

### PostgreSQL + Object Storage

**Use case:** Medium-to-large self-hosted deployments, VPS with external storage, managed hosting.

- Relational data in PostgreSQL.
- Media stored in an S3-compatible object store (AWS S3, MinIO, Backblaze B2, Cloudflare R2, etc.).
- Supports horizontal scaling of the application tier (multiple server processes behind a load balancer).
- sqlx with PostgreSQL driver in Rust.

**Configuration example:**
```toml
[storage]
backend = "postgres"
database_url = "postgresql://user:pass@localhost/decentcom"

[storage.media]
provider = "s3"
bucket = "decentcom-media"
region = "us-east-1"
# Uses standard AWS_ACCESS_KEY_ID / AWS_SECRET_ACCESS_KEY env vars
```

### Kubernetes-Native (Future)

**Use case:** Cloud-native operators, managed hosting infrastructure.

- Application runs as a Kubernetes Deployment with multiple replicas.
- PostgreSQL via a managed database (RDS, Cloud SQL, CockroachDB) or an in-cluster operator (CloudNativePG).
- Media via cloud object storage (S3, GCS, Azure Blob).
- Secrets managed via Kubernetes Secrets or a vault.

This is not a new storage backend so much as an operational configuration of the PostgreSQL + object storage backend. A Helm chart or Kubernetes manifests will be provided.

**Open question:** Should the server be able to use CockroachDB or other distributed SQL databases as a drop-in for PostgreSQL? This would allow global distribution without manual replication setup.

## Media Storage

All uploaded media (images, files, attachments) is stored as content-addressable blobs:
- The hash (SHA-256 or BLAKE3) of the file content is the key.
- Identical files are deduplicated automatically.
- A metadata record in the relational database maps `media_id → content_hash` with upload metadata (uploader, channel, timestamp, MIME type, size).

Media is served via the application server (which can set appropriate cache headers), or directly from the object store via pre-signed URLs (for S3 backends).

### Retention Policy

Servers can configure how long messages and media are retained:

| Policy | Behavior |
|---|---|
| `forever` | Messages and media are never deleted |
| `days: N` | Messages older than N days are soft-deleted; media purged after grace period |
| `count: N` | Only the N most recent messages per channel are retained |
| `manual` | No automatic deletion; admins manage retention |

Soft deletion marks records as deleted in the database but does not immediately remove them. A background job purges soft-deleted records and orphaned media on a configurable schedule.

## Backup and Restore

The server should support:
- **Full backup** — export all data as a portable archive (SQLite dump or PostgreSQL dump + media tar)
- **Incremental backup** — export changes since last backup
- **Restore** — import a backup archive to a fresh instance

This is essential for home server operators who want to migrate to a new machine or recover from failure.

**Open question:** Should we define a canonical interchange format (e.g. JSON + media zip) that allows migrating between storage backends? This would also be useful for importing from Discord (via the Discord data export feature).

## Session and Ephemeral State

Session tokens are short-lived (e.g. 24 hours). They should be stored in:
- An in-memory cache (acceptable for single-node; tokens are reissued on restart)
- Redis (for multi-node deployments where sessions need to be shared across instances)

Presence state (online/offline/typing) is entirely ephemeral and maintained in memory. It is not persisted. On server restart, all presence is cleared and clients re-establish their state on reconnect.

## Performance Considerations

- Message reads are the dominant workload. Indexes on `(channel_id, created_at)` are critical.
- The most recent N messages per channel should be efficiently fetchable (cursor-based pagination).
- Full-text search (if enabled) requires a search index. For SQLite: FTS5. For PostgreSQL: tsvector indexes. For large servers, an external search service (Meilisearch, Elasticsearch) may be preferable.
- Media serving should use a CDN or object storage direct URLs at scale, not the application server.
