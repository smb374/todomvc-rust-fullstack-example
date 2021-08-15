mod models;
mod schema;
use super::{Error, MyResult as Result, PgPool};
use diesel::{
    pg::PgConnection,
    prelude::*,
    r2d2::{ConnectionManager, Pool},
};
use models::Task;
use todomvc_shared::Entry;
use tokio_diesel::*;
use uuid::Uuid;

pub fn establish_connection() -> Result<PgPool> {
    let db = "postgres://<user>:<passwd>@<host>/<db>";
    let manager = ConnectionManager::<PgConnection>::new(db);
    let pool = Pool::builder().build(manager)?;
    Ok(pool)
}

pub async fn crate_task(pool: &PgPool, content: String) -> Result<Entry> {
    let task = Task::new(content);
    create_task_full(pool, &task).await.map(|_| task.to_entry())
}

pub async fn create_task_full(pool: &PgPool, t: &Task) -> Result<usize> {
    diesel::insert_into(schema::task::table)
        .values(t)
        .execute_async(pool)
        .await
        .map_err(Error::AsyncDiesel)
}

pub async fn get_tasks(pool: &PgPool) -> Result<Vec<Entry>> {
    schema::task::table
        .load_async(pool)
        .await
        .map(|v: Vec<Task>| v.iter().map(Task::to_entry).collect())
        .map_err(Error::AsyncDiesel)
}

pub async fn get_task(pool: &PgPool, task_id: Uuid) -> Result<Option<Entry>> {
    use schema::task::dsl::*;
    let tasks: Vec<Task> = task.filter(id.eq(task_id)).load_async(pool).await?;
    if tasks.len() > 1 {
        Err(Error::General(format!(
            "We uses uuid for our id, but we get {} tasks when querying id: {}.",
            tasks.len(),
            task_id
        )))
    } else if tasks.is_empty() {
        Ok(None)
    } else {
        let t = tasks[0].clone();
        let entry = t.to_entry();
        Ok(Some(entry))
    }
}

pub async fn update_all_tasks(pool: &PgPool, entries: Vec<Entry>) -> Result<()> {
    for entry in entries {
        let id = *entry.id();
        match get_task(pool, id).await {
            Ok(o) => match o {
                Some(_e) => update_task(pool, id, entry).await?,
                None => {
                    let task = Task::from(&entry);
                    create_task_full(pool, &task).await?
                }
            },
            Err(_) => {
                let task = Task::from(&entry);
                create_task_full(pool, &task).await?
            }
        };
    }
    Ok(())
}

pub async fn update_task(pool: &PgPool, eid: Uuid, e: Entry) -> Result<usize> {
    use schema::task::dsl::*;
    let t = Task::from(e);
    diesel::update(schema::task::table)
        .filter(id.eq(eid))
        .set((
            content.eq(t.content().to_owned()),
            completed.eq(*t.completed()),
            editing.eq(*t.editing()),
        ))
        .execute_async(pool)
        .await
        .map_err(Error::AsyncDiesel)
}

pub async fn remove_task(pool: &PgPool, eid: Uuid) -> Result<usize> {
    use schema::task::dsl::*;
    match get_task(pool, eid).await {
        Ok(o) => match o {
            Some(_e) => diesel::delete(schema::task::table)
                .filter(id.eq(eid))
                .execute_async(pool)
                .await
                .map_err(Error::AsyncDiesel),
            None => Err(Error::General("Task table is empty!".to_string())),
        },
        Err(e) => Err(Error::General(format!(
            "Cannot get such task with id: {}, reason: {}",
            eid.as_u128(),
            e.to_string()
        ))),
    }
}
