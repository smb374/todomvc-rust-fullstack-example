pub mod db;

use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use rocket::{
    delete, fs::NamedFile, get, http::Status, post, put, response::Redirect, routes,
    serde::msgpack::MsgPack, uri, Either, Route, State,
};
use std::{io, path::PathBuf};
use std::{path::Path, sync::Arc};
use thiserror::Error as TError;
use todomvc_shared::{Entries, Entry, TaskRequest, UpdateAll};
use uuid::Uuid;

pub type PgPool = Pool<ConnectionManager<PgConnection>>;
pub type MyResult<T> = Result<T, Error>;

#[derive(TError, Debug)]
pub enum Error {
    #[error("r2d2 encounters an error: {0}")]
    R2d2(#[from] r2d2::Error),
    #[error("IO error: {0}")]
    IO(#[from] io::Error),
    #[error("Diesel async error: {0}")]
    AsyncDiesel(#[from] tokio_diesel::AsyncError),
    #[error("Rocket error: {0}")]
    Rocket(#[from] rocket::Error),
    #[error("Error: {0}")]
    General(String),
}

pub fn all_routes() -> Vec<Route> {
    routes![
        index,
        static_files,
        create_task,
        get_task,
        update_task,
        get_tasks,
        update_all_tasks,
        delete_task
    ]
}

#[get("/")]
async fn index() -> Redirect {
    Redirect::to(uri!("/index.html"))
}

#[get("/<path..>")]
async fn static_files(path: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(path)).await.ok()
}

#[post("/task", format = "application/msgpack", data = "<task_req>")]
async fn create_task(
    pool: &State<Arc<PgPool>>,
    task_req: MsgPack<TaskRequest<'_>>,
) -> (Status, Either<MsgPack<Entry>, String>) {
    let content = task_req.content.to_string();
    match db::crate_task(pool.as_ref(), content).await {
        Ok(e) => (Status::Ok, Either::Left(MsgPack(e))),
        Err(e) => (Status::InternalServerError, Either::Right(e.to_string())),
    }
}

#[get("/tasks")]
async fn get_tasks(pool: &State<Arc<PgPool>>) -> (Status, Either<MsgPack<Entries>, String>) {
    match db::get_tasks(pool.as_ref()).await {
        Ok(v) => (Status::Ok, Either::Left(MsgPack(v))),
        Err(e) => (Status::InternalServerError, Either::Right(e.to_string())),
    }
}

#[post("/tasks", format = "application/msgpack", data = "<tasks>")]
async fn update_all_tasks(
    pool: &State<Arc<PgPool>>,
    tasks: MsgPack<UpdateAll>,
) -> (Status, String) {
    match db::update_all_tasks(pool.as_ref(), tasks.0).await {
        Ok(_) => (Status::Ok, "Acknowledged".to_string()),
        Err(e) => (Status::InternalServerError, e.to_string()),
    }
}

#[get("/task?<id>")]
async fn get_task(
    pool: &State<Arc<PgPool>>,
    id: Uuid,
) -> (Status, Either<MsgPack<Option<Entry>>, String>) {
    match db::get_task(pool.as_ref(), id).await {
        Ok(e) => (Status::Ok, Either::Left(MsgPack(e))),
        Err(e) => (Status::NotFound, Either::Right(e.to_string())),
    }
}

#[put("/task?<id>", format = "application/msgpack", data = "<task>")]
async fn update_task(
    pool: &State<Arc<PgPool>>,
    id: Uuid,
    task: MsgPack<Entry>,
) -> (Status, String) {
    match db::update_task(pool, id, task.0).await {
        Ok(_) => (Status::Ok, "Acknowledged".to_string()),
        Err(e) => (Status::NotFound, e.to_string()),
    }
}

#[delete("/task?<id>")]
async fn delete_task(pool: &State<Arc<PgPool>>, id: Uuid) -> (Status, String) {
    match db::remove_task(pool, id).await {
        Ok(_) => (Status::Ok, "Acknowledged".to_string()),
        Err(e) => (Status::NotFound, e.to_string()),
    }
}
