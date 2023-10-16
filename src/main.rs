#![allow(unused)]

use crate::models::tasks;
use actix_web::{get, App, HttpResponse, HttpServer, web, Error};
use liquid::model::Value;
use liquid::object;
use models::prelude::*;
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveValue, Database, DbBackend, QueryOrder};
use sqlx::query_as;
use sqlx::Column;
use std::{fs, task};
use std::task::Context;
use actix_web::web::get;
use sea_orm::ActiveValue::Set;
use sea_orm::EntityOrSelect;
use sea_orm::QueryTrait;
use sea_orm::QuerySelect;
/*use sea_orm::sea_query::ColumnSpec::Default;*/
use crate::models::tasks::Model;
use crate::tasks::ActiveModel;
use sea_orm::*;
use std::default::Default;




mod db;
mod models;


async fn insert_task(id: i64, description: &str) -> Result<(), actix_web::Error> {
    let mut db: DatabaseConnection =
        Database::connect("postgres://postgres:root@localhost/seaorm").await.unwrap();

    let new = tasks::ActiveModel {
        title: Set("tasks".to_owned()),
        description: Set(Some(description.to_owned())),
        ..Default::default()
    };

    let res = Tasks::insert(new)
        .exec(&db)
        .await
        .unwrap();

    println!("Inserted: last_insert_id = {}", res.last_insert_id);


    Ok(())
}

async fn find_task_by_id( id: i32) -> Result<Option<tasks::Model>, actix_web::Error> {
    let mut db: DatabaseConnection =
        Database::connect("postgres://postgres:root@localhost/seaorm").await.unwrap();


    let task = Tasks::find_by_id(800).one(&db).await.unwrap();

    println!("Found task: {:?}", task);
    Ok(task)
}



async fn update_task_title(id: i32, new_title: &str) -> Result<(), Error> {
    let db_url = "postgres://postgres:root@localhost/seaorm";


    let mut db: DatabaseConnection = match Database::connect(db_url).await {
        Ok(connection) => connection,
        Err(err) => {
            eprintln!("Failed to connect to the database: {:?}", err);
            return Err(actix_web::error::ErrorInternalServerError("Failed to connect to the database"));
        }
    };

    let new: Option<tasks::Model> = match Tasks::find_by_id(id).one(&db).await {
        Ok(task) => task,
        Err(err) => {
            eprintln!("Failed to find the task: {:?}", err);
            return Err(actix_web::error::ErrorInternalServerError("Failed to find the task"));
        }
    };


    let mut new_task: tasks::ActiveModel = new.unwrap().into();

    // Update name attribute
    new_task.title = Set(new_title.to_owned());

    // Update corresponding row in the database using the primary key value
    let new: tasks::Model = new_task.update(&db).await.unwrap();

    Ok(())
}


/*async fn delete_task( id: i64) -> Result<(), actix_web::Error> {
    let mut db: DatabaseConnection =
        Database::connect("postgres://postgres:root@localhost/seaorm").await.unwrap();

    let task_to_delete = Tasks::find_by_id(72).one(&db).await.unwrap();





    if let Some(task) = task_to_delete {
        let result = task.delete(&db).await.unwrap();
        println!("Deleted task with ID: {:?}", result);
    } else {
        println!("Task with ID 800 not found.");
    }

    let task = Tasks::find()
        .filter(tasks::Column::Id.eq(800))
        .one(&db)
        .await
        .unwrap();

    if let Some(task) = task {
        let result = task.delete(&db).await.unwrap();
        println!("Deleted task with ID {}: {:?}", id, result);
    } else {
        println!("Task with ID {} not found.", id);
    }

    Ok(())
}*/


async fn delete_task(id: i64) -> Result<(), actix_web::Error> {
    let mut db: DatabaseConnection =
        Database::connect("postgres://postgres:root@localhost/seaorm").await.unwrap();

    let mut task_to_delete = tasks::ActiveModel{
        id: ActiveValue::Set(1),
        .. Default::default()
    };

    // Use .delete()
    let result = task_to_delete.delete(&db).await.unwrap();
    println!("Deleted task with ID {}: {:?}", id, result);

    Ok(())
}



#[get("/")]
async fn get_tasks() -> Result<HttpResponse, actix_web::Error> {
    let mut db: DatabaseConnection =
        Database::connect("postgres://postgres:root@localhost/seaorm").await.unwrap();

    let filter_condition = tasks::Column::Title.eq("Task 4");

    let total_task = tasks::Entity::find().count(&db);

    println!("{:?}", total_task.await);



    let tasks: Vec<tasks::Model> = Tasks::find()
        .all(&db)
        .await
        .unwrap();


    for task in &tasks {
        println!("Task Title: {}", task.title);
        println!("Task Description: {:?}", task.description);
    }


    println!("{:?}",tasks);


    /*let new = tasks::ActiveModel {
        title: Set("tasks".to_owned()),
        ..Default::default()
    };

    let res = Tasks::insert(new)
        .exec(&db)
        .await
        .unwrap();

    println!("Inserted: last_insert_id = {}", res.last_insert_id);

    println!("{:?}",tasks);

    let new: Option<tasks::Model> = Tasks::find_by_id(res.last_insert_id)
        .one(&db)
        .await.unwrap();

    println!("New: {new:?}");

    let mut new: tasks::ActiveModel = new
        .unwrap()
        .into();

    new.title = Set("New Task".to_owned());

    let new: tasks::Model = new
        .update(&db)
        .await
        .unwrap();

    println!("Updated: {new:?}");

    let result = new
        .delete(&db)
        .await.
        unwrap();

    println!("Deleted: {result:?}");*/






    let mut context = liquid::Object::new();
    context.insert(
        "tasks".into(),
        liquid::model::Value::Array(
            tasks
                .into_iter()
                .map(|task| {
                    let mut task_map = liquid::Object::new();
                    task_map.insert(
                        "title".into(),
                        liquid::model::Value::scalar(task.title.to_string()),
                    );
                    task_map.insert(
                        "description".into(),
                        liquid::model::Value::scalar(
                            task.description.expect("Description").to_string(),
                        ),
                    );
                    liquid::model::Value::Object(task_map)
                })
                .collect(),
        ),
    );

    let template_source =
        fs::read_to_string("templates/tasks.html").expect("Failed to read the file");

    let parser = liquid::ParserBuilder::with_stdlib()
        .build()
        .map_err(|err| {
            actix_web::error::ErrorInternalServerError(format!("Failed to build parser: {}", err))
        })?;

    let template = parser
        .parse(&template_source)
        .expect("Failed to parse template");

    let output = template.render(&context).map_err(|err| {
        actix_web::error::ErrorInternalServerError(format!(
            "Failed to render the template: {}",
            err
        ))
    })?;

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(output))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {

    /*insert_task(1, "New Task").await;*/

    match find_task_by_id(4).await {
        Ok(Some(task)) => println!("Found task: {:?}", task),
        Ok(None) => println!("Task not found."),
        Err(_) => println!("An error occurred."),

    }



    update_task_title(73, "New Title").await;
    delete_task(73).await;

    HttpServer::new(|| App::new()
        .service(get_tasks))
        //.service(web::resource("/").to(get_tasks))
        .bind(("127.0.0.1",8080))?
        .run()
        .await
}
