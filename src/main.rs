use std::fmt::Display;
use argon2::Argon2;
use hex::encode;

use rocket::{
    fairing::{Info, Fairing, Kind},
    http::{Header, CookieJar, Cookie},
    request::{self, FromRequest, Outcome},
    response::{status::BadRequest, content::RawHtml, Redirect, },
    form::Form, serde::json::Json
};
use rocket_db_pools::{
    sqlx::{
        self,
        FromRow,
        query_as,
        mysql::MySqlRow,
        Row,
        query
    }, 
    Database,
    Connection
};

use rocket::{Request, Response};
use serde::{Deserialize, Serialize};

#[macro_use]
extern crate rocket;
extern crate rocket_db_pools;

#[derive(Database)]
#[database("mysql_test")]
pub struct ThreadsDatabase(sqlx::MySqlPool);

pub struct Cors;

#[rocket::async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info {
            name: "Cross-Origin-Resource-Sharing Fairing",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, PATCH, PUT, DELETE, HEAD, OPTIONS, GET",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}
#[derive(Deserialize, Serialize, Debug, FromForm, Clone)]
pub struct ExistingUser {
    user_id: i32,
}

impl FromRow<'_, MySqlRow> for ExistingUser {
    fn from_row(row: &'_ MySqlRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            user_id: row.try_get("id")?
        })
    }
}

#[async_trait]
impl<'r> FromRequest<'r> for &'r ExistingUser {
    type Error = std::convert::Infallible;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let user_result: &Option<ExistingUser> = req.local_cache_async(async {
            let db = req.guard::<&ThreadsDatabase>().await.succeeded().unwrap();
            if let Some(value) = req.cookies().get_private("user_id") {
                if let Ok(id) = value.value().parse::<i32>() {
                    if let Ok(res) = query("SELECT id FROM users WHERE id = ?").bind(id).execute(&**db).await {
                        println!("{:?}", res);
                        return Some(ExistingUser { user_id: id })
                    }
                }   
            }
            None
            //match req.cookies().get_private("user_id") {
            //    Some(value) => {
            //        let id_str = value.value();
            //        println!("{}", id_str);
            //        match id_str.parse::<i32>() {
            //            Ok(id) => {
            //                match query("SELECT id FROM users WHERE id = ?").bind(id).execute(&**db).await {
            //                    Ok(res) => {
            //                        println!("{:?}", res);
            //                        Some(ExistingUser { user_id: id })
            //                    },
            //                    Err(error) => {
            //                        println!("{}", error);
            //                        None
            //                    }
            //                }
            //            },
            //            Err(error) => {
            //                println!("{}", error);
            //                None
            //            }
            //        }
            //    },
            //    None => {None}
            //}
        }).await;
        match user_result {
            Some(value) => Outcome::Success(value),
            None => Outcome::Forward(())
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thread {
    pub floss: i32,
    pub amount: i32,
    pub name: String,
    pub color: String,
}

impl FromRow<'_, MySqlRow> for Thread {
    fn from_row(row: &'_ MySqlRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            floss: row.try_get("floss")?,
            amount: row.try_get("amount")?,
            name: row.try_get("name")?,
            color: row.try_get("color")?,
        })
    }
}

#[derive(Deserialize, Serialize, Debug, FromForm, Copy, Clone)]
pub struct NewUser<'r> {
    username: &'r str,
    password: &'r str
}

impl <'r> Display for NewUser<'r> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Username: {}, Password: {}", self.username, self.password)
    }
}

impl <'r> From<NewUser<'r>> for String {
    fn from(value: NewUser) -> Self {
        format!("{{\"data\": \"username: {}, password: {}\"}}", value.username, value.password)
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum UpdateThreadOptions {
    Increment = 0,
    Decrement = 1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateThreadMessage {
    floss: i32,
    name: String,
    color: String,
    action: UpdateThreadOptions
}



#[options("/<_..>")]
fn all_options() {
    // Intentionally left empty 
}

#[catch(404)]
fn not_found(_: &Request) -> &'static str {
    "404 funny not found"
}

#[catch(500)]
fn oops(_: &Request) -> &'static str {
    "500: Oops all server error"
}

#[get("/")]
async fn index(_user: &ExistingUser) -> RawHtml<String> {
    RawHtml(include_str!("../main_page.html").to_string())
}

#[get("/", rank = 2)]
async fn login_redirect() -> Redirect {
    Redirect::to("/login")
}

#[get("/login")]
async fn login_page() -> RawHtml<String> {
    RawHtml(include_str!("../login.html").to_string())
}

#[post("/login", data="<data>")]
async fn login(data: Form<NewUser<'_>>, cookies: &CookieJar<'_>, mut db: Connection<ThreadsDatabase>) -> Redirect {
    let username = data.username.clone();
    let password = data.password.clone();
    let hashed_password = hash_password_to_string(password);
    match query_as::<_, ExistingUser>("SELECT id FROM users WHERE username = ? AND password = ?").bind(username).bind(hashed_password).fetch_one(&mut *db).await {
        Ok(value) => {
            cookies.add_private(Cookie::new("user_id", value.user_id.to_string()));
            Redirect::to("/")
        },
        Err(error) => {
            println!("login error: {}", error.to_string());
            Redirect::to("/login")
        }
    }
}

#[post("/signup", data="<data>")]
async fn signup(data: Form<NewUser<'_>>, mut db: Connection<ThreadsDatabase>, cookies: &CookieJar<'_>) -> Result<Redirect, BadRequest<String>> {
    let username = data.username.clone();
    let password = data.password.clone();
    let hashed_password = hash_password_to_string(password);
    println!("hashed_password: {}", hashed_password);
    match query("INSERT INTO users (username, password) VALUES (?, ?)")
        .bind(username)
        .bind(hashed_password)
        .execute(&mut *db)
        .await 
    {
        Ok(value) => {
            println!("signup mySqlQueryResult: {:?}", value);
            match query_as::<_, ExistingUser>("SELECT id FROM users WHERE username = ?").bind(username).fetch_one(&mut *db).await {
                Ok(user) => {
                    cookies.add_private(Cookie::new("user_id", user.user_id.to_string()));
                    Ok(Redirect::to("/"))
                },
                Err(error) => {
                    Err(BadRequest(Some(format!("select user id error: {}", error.to_string()))))
                }
            }
        },
        Err(error) => {
            Err(BadRequest(Some(format!("signup error: {}", error.to_string()))))
        }
    }
}

#[post("/signout")]
async fn signout_post(_user: &ExistingUser, cookies: &CookieJar<'_>) -> String {
    cookies.remove_private(Cookie::named("user_id"));
    format!("Successfully signed out")
}
#[get("/signout")]
async fn signout_get(_user: &ExistingUser, cookies: &CookieJar<'_>) -> String {
    cookies.remove_private(Cookie::named("user_id"));
    format!("Successfully signed out")
}

#[get("/getthreads")]
async fn get_threads(mut db: Connection<ThreadsDatabase>, user: &ExistingUser) -> Result<Json<Vec<Thread>>, BadRequest<String>> {
    match query_as::<_, Thread>("SELECT floss, amount, name, color FROM threads WHERE userId = ?").bind(user.user_id).fetch_all(&mut *db).await {
        Ok(value) => {
            Ok(Json(value))
        },
        Err(error) => {
            Err(BadRequest(Some(error.to_string())))
        }
    }
}

#[post("/updatethread", data = "<data>")]
async fn update_thread(mut db: Connection<ThreadsDatabase>, user: &ExistingUser, data: Json<UpdateThreadMessage>) -> Result<String, BadRequest<String>> {
    let action = if data.action == UpdateThreadOptions::Increment {true} else {false};
    match query("CALL ModifyThreadAmount(?, ?, ?, ?, ?);").bind(user.user_id).bind(data.floss).bind(data.name.clone()).bind(data.color.clone()).bind(action).execute(&mut *db).await {
        Ok(res) => Ok(format!("{res:#?}")),
        Err(err) => Err(BadRequest(Some(err.to_string())))

    }
    //let operator = match data.action {
    //    UpdateThreadOptions::Increment => "+",
    //    UpdateThreadOptions::Decrement => "-"
    //};
    //let statement = format!("UPDATE threads SET amount = amount {} 1 WHERE userId = ? AND floss = ?", operator);
    //match query_as::<_, Thread>("SELECT * FROM threads WHERE userId = ? AND floss = ?").bind(user.user_id).bind(data.floss).fetch_optional(&mut *db).await {
    //    Ok(value) => {
    //        match value {
    //            Some(thread) => {
    //                match (thread.amount, data.action) {
    //                    (1, UpdateThreadOptions::Decrement) => {
    //                        match sqlx::query("DELETE FROM threads WHERE userId = ? AND floss = ?").bind(user.user_id).bind(data.floss).execute(&mut *db).await {
    //                            Ok(value) => {
    //                                Ok(format!("{value:?}"))
    //                            },
    //                            Err(error) => {
    //                                Err(BadRequest(Some(error.to_string())))
    //                            }
    //                        }
    //                    },
    //                    (_, _) => {
    //                        match sqlx::query(&statement).bind(user.user_id).bind(data.floss).execute(&mut *db).await {
    //                            Ok(value) => {
    //                                Ok(format!("{value:?}"))
    //                            },
    //                            Err(error) => {
    //                                Err(BadRequest(Some(error.to_string())))
    //                            }
    //                        }
    //                    },
    //                }
    //            },
    //            None => {
    //                println!("No thread");
    //               match data.action {
    //                UpdateThreadOptions::Increment => {
    //                    match sqlx::query("INSERT INTO threads (userId, floss, amount) VALUES (?, ?, 1)").bind(user.user_id).bind(data.floss).execute(&mut *db).await {
    //                        Ok(value) => {
    //                            Ok(format!("{value:?}"))
    //                        },
    //                        Err(error) => {
    //                            Err(BadRequest(Some(error.to_string())))
    //                        }
    //                    }
    //                },
    //                UpdateThreadOptions::Decrement => {
    //                    Err(BadRequest(Some(String::from("Unable to decrement a non-existent thread"))))
    //                } 
    //               }
    //            }
    //        }
    //    },
    //    Err(error) => {
    //        Err(BadRequest(Some(error.to_string())))
    //    }
    //}
}

#[post("/updatethreads", data="<data>")]
async fn update_threads(mut db: Connection<ThreadsDatabase>, user: &ExistingUser, data: Json<Vec<Thread>>) {
    let mut sql: String = "INSERT INTO threads (userId, floss, amount, name, color) VALUES".to_owned();
    for i in 0..data.len() {
        let thread = data[i].to_owned();
        sql.push_str(format!("({}, {}, {}, \"{}\", \"{}\")", user.user_id, thread.floss, thread.amount, thread.name, thread.color).as_str());
        if i + 1 != data.len() {
            sql += ",";
        }
    }
    sql += " AS aliased ON DUPLICATE KEY UPDATE amount = aliased.amount;";
    println!("{sql}");
    match query(&sql).execute(&mut *db).await {
        Ok(res) => {
            println!("{:#?}", res);
        },
        Err(err) => {
            println!("{:#?}", err);
        }
    }
}


#[get("/testhtml")]
async fn test_html() -> RawHtml<String> {
    println!("test html");
    RawHtml(String::from(include_str!("../test.html")))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default, Responder)]
#[response(status = 200, content_type = "image/x-icon")]
struct Favicon(&'static [u8]);

#[get("/favicon.ico")]
async fn favicon() -> Favicon {
    Favicon(include_bytes!("../favicon.ico"))
}


#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(ThreadsDatabase::init())
        .attach(Cors)
        .mount("/", routes![all_options, index, login_page, login, signup, get_threads, test_html,  update_thread, login_redirect, favicon, signout_get, signout_post, update_threads])
        .register("/", catchers![not_found, oops])
}

fn hash_password_to_string(password: &str) -> String {
    let salt = format!("{password}_salt_lol");
    let mut output_password = [0u8; 32];
    Argon2::default().hash_password_into(
      password.as_bytes(),
      salt.as_bytes(),
      &mut output_password
    ).expect("unable to hash password");
    encode(output_password)
  }