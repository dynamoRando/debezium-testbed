use mysql::{prelude::Queryable, Params, Pool};
use anyhow::Result;

const URL: &str = "mysql://root:testbed@localhost:23306/testbed";

pub fn get_mysql_pool() -> Result<Pool> {
    Ok(Pool::new(URL)?)
}

/// Drops the specified database if it exists and creates it
pub fn create_db_forcefully(db_name: &str) {
    let pool = get_mysql_pool().unwrap();
    let mut conn = pool.get_conn().unwrap();

    let sql = format!("DROP DATABASE IF EXISTS {};", db_name);
    conn.exec_drop(&sql, Params::Empty).unwrap();
    let sql = format!("CREATE DATABASE {};", db_name);
    conn.exec_drop(&sql, Params::Empty).unwrap();
}
