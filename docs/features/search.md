# Feature: Message Search

## Overview
Provide full-text search across messages within a server. Users can search by keyword, with results showing matching messages in context. The feature uses SQLite FTS5 for the default storage backend and is gated behind the `message_search` feature flag so operators can disable it on resource-constrained servers.

## Background
The storage doc (`docs/design/storage.md`) notes that full-text search requires a search index: FTS5 for SQLite, tsvector for PostgreSQL, or an external service (Meilisearch, Elasticsearch) for large servers. The server model (`docs/design/server-model.md`) lists `message_search` as a feature flag (enabled by default) with the note that it "may be expensive for large servers." The initial implementation targets SQLite FTS5 only; PostgreSQL and external search are future work.

## Requirements
- [ ] Users can search messages by keyword across all channels they have `VIEW_CHANNEL` permission for
- [ ] Search results include the matching message, its author, channel, and timestamp
- [ ] Search results are paginated (cursor-based, consistent with message history pagination)
- [ ] The `message_search` feature flag gates the search endpoint; when disabled, returns 403
- [ ] Search is scoped to the current server (no cross-server search)
- [ ] Users can optionally filter search by channel, author, and date range
- [ ] The FTS index is kept in sync with message creates, edits, and deletes
- [ ] Search queries support basic operators: quoted phrases for exact match, prefix matching
- [ ] Soft-deleted messages are excluded from search results
- [ ] The client provides a search UI accessible from the channel view

## Design

### API / Interface Changes

**New REST endpoint:**

`GET /api/v1/search`

Query parameters:
- `q` (required) — Search query string. Passed to FTS5 `MATCH` expression.
- `channel_id` (optional) — Filter to a specific channel.
- `author_id` (optional) — Filter by message author.
- `before` (optional) — ISO 8601 timestamp, only messages before this time.
- `after` (optional) — ISO 8601 timestamp, only messages after this time.
- `limit` (optional, default 25, max 50) — Number of results per page.
- `cursor` (optional) — Pagination cursor from previous response.

Response:
```json
{
  "results": [
    {
      "message": { /* full message object with attachments, reactions */ },
      "channel": { "id": "...", "name": "general" },
      "highlight": "...matched **keyword** in context..."
    }
  ],
  "cursor": "next_page_cursor_or_null",
  "total_estimate": 42
}
```

The endpoint checks `VIEW_CHANNEL` permission for each result channel, filtering out messages from channels the requester cannot access.

**New Tauri IPC command:**

`search_messages(server_id, query, filters)` — Calls the search API and returns results to the React frontend.

### Data Model Changes

**New FTS5 virtual table (SQLite):**

```sql
CREATE VIRTUAL TABLE messages_fts USING fts5(
    content,
    content='messages',
    content_rowid='rowid',
    tokenize='porter unicode61'
);

-- Triggers to keep FTS in sync
CREATE TRIGGER messages_fts_insert AFTER INSERT ON messages BEGIN
    INSERT INTO messages_fts(rowid, content) VALUES (new.rowid, new.content);
END;

CREATE TRIGGER messages_fts_delete AFTER DELETE ON messages BEGIN
    INSERT INTO messages_fts(messages_fts, rowid, content) VALUES ('delete', old.rowid, old.content);
END;

CREATE TRIGGER messages_fts_update AFTER UPDATE OF content ON messages BEGIN
    INSERT INTO messages_fts(messages_fts, rowid, content) VALUES ('delete', old.rowid, old.content);
    INSERT INTO messages_fts(rowid, content) VALUES (new.rowid, new.content);
END;
```

The `porter` tokenizer provides stemming (e.g., "running" matches "run"). The `unicode61` tokenizer handles Unicode normalization.

No new relational tables are needed. The FTS table is a virtual index over the existing `messages` table.

### Component Changes

**Server (`server/`):**
- `server/src/routes/search.rs` — New module: search endpoint with query parsing, FTS5 query execution, permission filtering, pagination
- `server/src/models/search.rs` — New module: `SearchResult` struct, FTS query builder that translates user input into safe FTS5 match expressions
- `server/src/storage/sqlite/migrations/` — Migration to create FTS5 virtual table and triggers
- `server/src/storage/sqlite/messages.rs` — Ensure message insert/update/delete operations work with FTS triggers (no code change if using raw SQL; triggers handle it)

**Client (`client/`):**
- `client/src/components/SearchPanel.tsx` — New component: search input, filter controls (channel, author, date range), results list with highlighted snippets
- `client/src/components/SearchResult.tsx` — New component: renders a single search result with message preview, channel name, author, timestamp, and a "Jump to message" link
- `client/src/hooks/useSearch.ts` — New hook: manages search state, debounced query submission, pagination
- `client/src/api/search.ts` — API client function for the search endpoint
- `client/src/components/ChannelHeader.tsx` — Add search icon/button that toggles `SearchPanel`

## Task List

### Server
- [ ] Add FTS5 virtual table and sync triggers to SQLite migrations (`server/migrations/009_fts5.sql`)
- [ ] FTS5 query sanitization in `server/src/storage/sqlite/search.rs` (`sanitize_fts_query`): handles quoted phrases, strips special chars from bare words
- [ ] `SearchStore` trait and SQLite impl in `server/src/storage/sqlite/search.rs`
- [ ] `GET /api/v1/search` endpoint in `server/src/search/handlers.rs`
- [ ] Channel permission filtering: user's accessible channels queried via `RoleStore`, included in SQL WHERE
- [ ] Optional `channel_id` filter supported
- [ ] Gate endpoint behind `message_search` feature flag check
- [ ] Backfill existing messages into FTS index (migration step with INSERT INTO messages_fts)

### Client
- [ ] Add search API client in `client/src/api/search.ts`
- [ ] Create `useSearch` hook with debounced query (300ms), pagination state, and loading/error state
- [ ] Create `SearchPanel.tsx` with search input, filter dropdowns, and results area
- [ ] Create `SearchResult.tsx` with highlighted snippet, channel badge, author, timestamp
- [ ] Add "Jump to message" functionality: navigate to the channel and scroll to the message
- [ ] Add search toggle button to `ChannelHeader.tsx`
- [ ] Style search panel with Catppuccin theme, slide-in from right or overlay panel
- [ ] Handle empty state, no results, and error states

## Test List
- [ ] Unit test: FTS query sanitizer correctly handles quoted phrases and strips special chars (`storage/sqlite/search.rs`)
- [ ] Unit test: sanitizer returns None for empty/whitespace-only queries
- [ ] Integration test: insert messages, search by keyword, verify correct results returned (`routes/search_tests.rs`)
- [ ] Integration test: search excludes soft-deleted messages
- [ ] Integration test: search with channel filter returns only messages from that channel
- [ ] Integration test: search endpoint returns 403 when `message_search` feature flag is disabled
- [ ] Integration test: empty query returns 422
- [ ] Integration test: search excludes messages from channels the user cannot access
- [ ] Integration test: editing a message updates the FTS index (new content is searchable)
- [ ] Manual: open search panel, type a query, verify results appear with highlighted matches
- [ ] Manual: click "Jump to message" on a search result, verify navigation to correct channel and message
- [ ] Manual: apply filters (channel, author), verify results narrow correctly

## Open Questions
- Should search rank by relevance (FTS5 `rank`) or by recency (newest first)? Discord uses recency. Consider offering both via a sort parameter.
- Should we support boolean operators (`AND`, `OR`, `NOT`) in the query syntax, or keep it simple with just keywords and quoted phrases?
- How should the backfill migration work for servers with large existing message histories? A batched approach is needed to avoid locking the database.
- Should the `total_estimate` be an exact count or an approximation? Exact counts on FTS queries can be expensive.
- For the PostgreSQL backend (future), should we use `tsvector`/`tsquery` or defer to an external search service?
