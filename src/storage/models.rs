//! Models can be called by other crates, not just storage crate. They abstract
//! away the process of querying the database. The actual query building all
//! happens in the storage singleton service. The storage service holds any
//! active session tokens to access the tenant databases. So, keeping the SQL
//! isolated within the storage service prevents a different compromised
//! process (for instance a runtime process) running arbitrary queries or
//! accessing users' data other than its own.

pub mod graph;
pub mod node;
pub mod user;
pub mod sync;

pub mod graph_impl;
pub mod node_impl;
pub mod user_impl;
pub mod sync_impl;
