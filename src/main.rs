#![feature(proc_macro_hygiene, decl_macro)]
#[warn(deprecated)]
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;
extern crate env_logger;
extern crate r2d2;
extern crate r2d2_sqlite;
pub extern crate rusqlite;
extern crate serde_json;
#[macro_use]
extern crate lazy_static;

use r2d2::{Pool, PooledConnection};
use rocket::fairing::AdHoc;
use rocket::{
    response::{content, Debug},
    routes, Rocket, State,
};
use rocket_contrib::databases;
use rocket_contrib::json::Json;
use rocket_contrib::serve::StaticFiles;
use rusqlite::{types::FromSql, types::ToSql, Connection, MappedRows, Result, Row};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use std::marker::Sync;
use std::sync::Mutex;
use std::sync::RwLock;

#[database("SqliteDbConn")]
struct SqliteDbConn(Connection);

#[derive(Debug, Serialize, Deserialize)]
struct Movies {
    movie_id: i32,
    title: String,
    genre: String,
    imdb_rating: f64,
}
#[derive(Debug, Serialize, Deserialize)]
struct Datas {
    all_movies: Vec<Movies>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Tv_series {
    series_id: i32,
    title: String,
    genre: String,
    season: u64,
    episode: u64,
    imdb_rating: i32,
}

fn read_sql_from_file(path: &str) -> String {
    let mut file = File::open(path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    contents
}

fn create_db(conn: &Connection, sql_content: String) -> Result<usize> {
    conn.execute("DROP TABLE IF EXISTS Movies", &[])?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS Movies (
    movie_id    INTEGER PRIMARY KEY AUTOINCREMENT,
    title       TEXT NOT NULL,
    genre       TEXT NOT NULL,
    imdb_rating INTEGER NOT NULL
);
",
        &[],
    )?;

    conn.execute(
        "INSERT INTO Movies VALUES (1, \"chalo\", \"action\", 4.5 )",
        &[],
    )?;

    conn.execute(
        "INSERT INTO Movies VALUES (2, \"chale\", \"scifi\", 4.5 )",
        &[],
    )
}

fn my_movies(conn: &Connection) -> Result<Json<Datas>> {
    let mut stmt = conn
        .prepare("SELECT * FROM Movies")
        .expect("Movies not found");
    let mut all_movies = stmt
        .query_map(&[], |row| Movies {
            movie_id: row.get(0),
            title: row.get(1),
            genre: row.get(2),
            imdb_rating: row.get(3),
        })
        .unwrap()
        .into_iter()
        .collect::<Result<Vec<Movies>>>()?;

    println!("{:?}", all_movies);

    Ok(Json(Datas { all_movies }))
}

#[get("/movies")]
fn get_movies(conn: SqliteDbConn) -> Result<Json<Datas>> {
    println!("hey");

    my_movies(&conn)
    //let movies_json = serde_json::to_string(&movies_iter).unwrap();
    //println!("{}", movies_json);
}

fn run_migrations(rocket: Rocket) -> std::result::Result<Rocket, Rocket> {
    let sql_file_content = read_sql_from_file("all.sql");
    let conn = SqliteDbConn::get_one(&rocket).expect("db conn");
    create_db(&conn, sql_file_content).expect("as");
    println!("done migr");
    Ok(rocket)
}

fn main() {
    env_logger::init();

    rocket::ignite()
        .attach(SqliteDbConn::fairing())
        .attach(AdHoc::on_attach("Migration", run_migrations))
        .mount("/", routes![get_movies])
        //.manage(Mutex::new(db_conn))
        .launch();
}
