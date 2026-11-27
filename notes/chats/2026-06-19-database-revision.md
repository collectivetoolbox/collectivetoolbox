# Storage Backend Transition: Redb to Turso (libsql)

This plan evaluates the viability of migrating the `ctb-storage` crate's persistence layer from `redb` to `turso` (libsql/limbo). It outlines the architectural simplifications, design adjustments, and a phase-by-phase roadmap to transition user registry data, unblock multithreaded tests, and enable storage-dependent features.

---

## Viability Evaluation

We compared the characteristics of `redb` (with custom file-locking/envelope-encryption wrapper) against `turso` (libsql):

| Criteria | Old Design (Redb + Custom Wrapper) | New Design (Turso / libsql) | Viability / Impact |
| :--- | :--- | :--- | :--- |
| **Concurrency** | Single-writer limit; custom flaky `ResourceLock` file-locking wrapper that deadlocks in async/multithreaded tokio tests. | Built-in SQLite-style reader/writer locking and MVCC. Multi-process WAL coordination. | **High Viability**: Resolves all thread/process deadlocks; tests can run multithreaded. |
| **Storage Layout** | Multiple separate `.redb` files on disk for each table/entity (e.g. `ids.redb`, `auth.redb`). | A single SQLite database file (e.g., `users.db`) containing standard relational tables. | **Better**: Simpler filesystem footprint; strong foreign key and transaction consistency. |
| **Encryption-at-Rest** | Manual envelope encryption of 32MB shards using `rage` crate + zstd compression + padding. | Native page-level database encryption (`experimental_encryption` & `with_encryption(opts)`). | **High Viability**: Delegated entirely to database engine; eliminates envelope encryption code. |
| **Fragmentation** | Splitting databases into 32MB files to limit memory/disk writes and ease cloud offloading. | Single database file per user containing all graphs/cached data. Optional second archive DB attached via SQL. | **Better**: Avoids complex 2PC transaction logic and fragmentation overhead. |
| **Indexing** | Roaring bitmaps stored in separate index files, requiring custom set-operation algebra. | Native SQLite SQL indexes, index queries, and FTS5 (Full-Text Search). | **Better**: Standard SQL querying replacing custom bitmap intersection logic. |
| **Synchronization** | Custom Merkle-manifest-based object syncing. | Built-in local-first Turso syncing / replication client. | **High Viability**: Built-in synchronization is a core feature of the Turso client. |

### Conclusion
Transitioning to Turso is **highly viable** and strongly recommended. It addresses the core locking issues that currently force single-threaded tests, simplifies the storage model, and delegates complex operations (indexing, transaction safety, replication, and encryption) to a proven database engine.

---

## Key Design Adjustments

### 1. Consolidated Registry Database (Unencrypted)
Instead of separate `.redb` databases, we will use a single SQLite file named `users.db` under the user storage directory. The database itself will remain unencrypted at rest; only the user's private credentials (wrapped DEK and KEK parameters) will be stored in encrypted form inside the database, exactly as at present.

```sql
CREATE TABLE IF NOT EXISTS users (
    user_id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    uuid BLOB NOT NULL UNIQUE,
    phc_hash TEXT NOT NULL,
    display_name BLOB,
    picture_data BLOB,
    kek_params BLOB NOT NULL,
    wrapped_dek BLOB NOT NULL,
    pubkey BLOB
);
```

### 2. Elimination of `ResourceLock` for DB Operations
With Turso's concurrent access capabilities, we will completely remove the database-level `ResourceLock` file locks from `db_impl.rs`. Standard SQL transactions (`BEGIN TRANSACTION`, `COMMIT`) will guarantee atomicity and thread-safety.

### 3. Consolidated Single-File DB per User (Encrypted)
To prevent behavior and data volume leakage patterns:
- All of a user's local graphs—including their unshareable primary graph, imported team/shared graphs, and cached global graph data—will be stored within a **single** local database file `user_data.db` (under `graphs/<user_id>/user_data.db`).
- This entire file will be encrypted using Turso's native page-level encryption, initialized with the decrypted user DEK as the hexkey option via `.with_encryption(EncryptionOpts { cipher: "aes256", hexkey: "..." })`.
- Because all local and cached global data resides within a single encrypted file, an observer sees only one opaque file growing in size, preventing any leakage of private data ratios or behavior patterns.

### 4. Partitioning & Offloading (Overlay DBs)
To support large data volumes and offloading to external partitions:
- We can support attaching a second large archive database file (e.g., `user_data_archive.db` located on a second partition or external drive) using SQLite's native `ATTACH DATABASE` feature at runtime:
  ```sql
  ATTACH DATABASE '/path/to/archive/user_data_archive.db' AS archive KEY 'decrypted_dek_hex';
  ```
- Both active and archive databases share identical schema tables (e.g., `triples` and `nodes`). The application overlays them transparently using unified views:
  ```sql
  CREATE TEMP VIEW unified_triples AS
  SELECT * FROM main.triples
  UNION ALL
  SELECT * FROM archive.triples;
  ```
- Graph sharing to other users is done at the application layer by exporting the relevant graph triples, encrypting them under the shared Graph Key (GK), and transmitting them to the recipient, who then imports them into their own local `user_data.db`.

---

## Proposed Changes

### Storage Component

#### [MODIFY] [Cargo.toml](ctoolbox/src/storage/Cargo.toml)
- Ensure the `turso` dependency is active with necessary features.
- Remove `redb` and `fs2` dependencies once migration is complete.

#### [MODIFY] [db_impl.rs](ctoolbox/src/storage/db_impl.rs)
- Remove `redb` imports.
- Initialize and pool Turso `Database` and `Connection` objects instead of `redb::Database`.
- Rewrite the `#[ipc_method]` database query methods (`get_str_u64`, `put_str_u64`, etc.) to run SQL queries against the pooled `users.db` connection.
- Remove the cross-process `ResourceLock` helper wrappers for databases.

#### [MODIFY] [user.rs](ctoolbox/src/storage/user.rs)
- Simplify user registration and login by removing the `ResourceLock` blocks around DB calls.
- Utilize the consolidated SQLite `users.db` tables.

---

## Verification Plan

### Automated Tests
- Run `cargo test -p ctb-storage` multithreaded (remove `-- --test-threads=1` constraint if configured in workspace test runner).
- Verify the user auth controller tests: `cargo test --package ctb-io --lib io::webui::controllers::auth::auth_controller_tests`.

### Manual Verification
- Run the web UI locally (`npm run dev`) and test the registration and login flows to ensure correct sessions and redirection to `/home`.

