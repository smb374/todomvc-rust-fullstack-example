#[macro_use]
extern crate diesel;

mod lib;
use lib::{all_routes, db, PgPool};

use std::sync::Arc;

#[rocket::main]
async fn main() -> lib::MyResult<()> {
    let db: PgPool = db::establish_connection()?;
    let db_arc: Arc<PgPool> = Arc::new(db);
    rocket::build()
        .mount("/", all_routes())
        .manage(db_arc)
        .launch()
        .await
        .map_err(lib::Error::Rocket)
}
