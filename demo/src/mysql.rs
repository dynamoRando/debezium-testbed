use mysql::{prelude::Queryable, Params, Pool};
use tracing::info;

pub fn get_mysql_pool(db_name: &str) -> Pool {
    let url = format!("mysql://root:testbed@localhost:23306/{}", db_name);
    let url = url.as_str();
    Pool::new(url).unwrap()
}

pub fn add_test_data(pool: Pool, num: u32) {    
    info!("adding test data with value: {num:?}");
    let sql = format!("INSERT INTO example (id) VALUES ({});", num);
    let mut conn = pool.get_conn().unwrap();
    conn.exec_drop(sql, Params::Empty).unwrap();
}