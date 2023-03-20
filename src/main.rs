
use rocket_db_pools::sqlx::{self, Row};
use rocket_db_pools::{Database, Connection};

#[macro_use]
extern crate rocket;

use rocket::Request;

#[derive(Database)]
#[database("mysql_test")]
struct Test(sqlx::MySqlPool);

//#[derive(Database)]
//#[database("test")]
//struct TestDB(Client);

//#[get("/test")]
//async fn test(mut db: Connection<TestDB>) {
//    for db_name in db.list_database_names(None, None).await.unwrap() {
//        println!("{}", db_name);
//    }
//}


#[get("/")]
fn index() -> &'static str {
    "Hello, from Rocket!"
}

#[catch(404)]
fn not_found(req: &Request) -> &'static str {
    "Fuck off"
}

#[catch(500)]
fn oops(req: &Request) -> &'static str {
    "Oops we fucked up lol. better luck next time bozo"
}

#[get("/id/<id>")]
async fn create_table(mut db: Connection<Test>, id: i64) -> Option<String> {
    let row = sqlx::query("SELECT username FROM users WHERE id = ?").bind(id).fetch_one(&mut *db).await;
    match row {
        Ok(x) => x.try_get(0).ok(),
        Err(_) => None
    }

}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Test::init())
        .mount("/", routes![index, create_table])
        .register("/", catchers![not_found, oops])
}
