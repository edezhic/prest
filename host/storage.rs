use anyhow::Result;
pub use gluesql::core::ast_builder::table;
use gluesql::{core::ast_builder::Build, prelude::*};

lazy_static! {
    static ref STORE: SharedMemoryStorage = SharedMemoryStorage::new();
}
pub static SESSION_TABLE: &str = "Sessions";

pub fn init() -> Result<()> {
    exec_inside_sync(
        table(SESSION_TABLE)
            .create_table_if_not_exists()
            .add_column("user TEXT PRIMARY KEY")
            .add_column("session_id UINT128")
            .add_column("session_ts TIMESTAMP"),
    )?;
    Ok(())
}

pub fn exec_inside_async(stmt: impl Build) -> Result<Payload> {
    // temporary workaround until Glue futures implement Send https://github.com/gluesql/gluesql/issues/1245
    Ok(tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(exec(stmt))
    })?)
}

pub fn exec_inside_sync(stmt: impl Build) -> Result<Payload> {
    // minimalistic async executor
    Ok(futures::executor::block_on(exec(stmt))?)
}

async fn exec(stmt: impl Build) -> Result<Payload> {
    Ok(Glue::new(STORE.clone())
        .execute_stmt(&stmt.build()?)
        .await?)
}
