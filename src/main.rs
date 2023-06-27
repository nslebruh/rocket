use std::{fmt::Display, path::{PathBuf, Path}, io};
use argon2::Argon2;
use hex::encode;
use std::fs;

use rocket::{
    fairing::{Info, Fairing, Kind},
    http::{Header, CookieJar, Cookie},
    request::{self, FromRequest, Outcome},
    response::{status::BadRequest, content::RawHtml, Redirect},
    form::Form, fs::NamedFile, serde::json::Json
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
            user_id: row.try_get("UserId")?
        })
    }
}

#[async_trait]
impl<'r> FromRequest<'r> for &'r ExistingUser {
    type Error = std::convert::Infallible;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let user_result: &Option<ExistingUser> = req.local_cache_async(async {
            let db = req.guard::<&ThreadsDatabase>().await.succeeded().unwrap();
            match req.cookies().get_private("user_id") {
                Some(value) => {
                    let id_str = value.value();
                    println!("{}", id_str);
                    match id_str.parse::<i32>() {
                        Ok(id) => {
                            match query("SELECT UserId FROM users WHERE UserId = ?").bind(id).execute(&**db).await {
                                Ok(res) => {
                                    println!("{:?}", res);
                                    Some(ExistingUser { user_id: id })
                                },
                                Err(error) => {
                                    println!("{}", error);
                                    None
                                }
                            }
                        },
                        Err(error) => {
                            println!("{}", error);
                            None
                        }
                    }
                },
                None => {None}
            }
        }).await;
        match user_result {
            Some(value) => {
                Outcome::Success(value)
            },
            None => {
                Outcome::Forward(())
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Thread {
    #[serde(rename = "floss")]
    pub floss: i32,
    #[serde(rename = "amount")]
    pub amount: i32,
}

impl FromRow<'_, MySqlRow> for Thread {
    fn from_row(row: &'_ MySqlRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            floss: row.try_get("Floss")?,
            amount: row.try_get("Amount")?
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

#[options("/<_..>")]
fn all_options() {
    /* Intentionally left empty */
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
async fn index(_user: &ExistingUser) -> io::Result<NamedFile> {
    NamedFile::open("main_page.html").await
}

#[get("/", rank = 2)]
async fn login_redirect() -> Redirect {
    Redirect::to("/login")
}

#[get("/login")]
async fn login_page() -> io::Result<NamedFile> {
    println!("{}", include_str!("login.html"));
    NamedFile::open("login.html").await
}

#[post("/login", data="<data>")]
async fn login(data: Form<NewUser<'_>>, cookies: &CookieJar<'_>, mut db: Connection<ThreadsDatabase>) -> Result<Redirect, BadRequest<String>> {
    let username = data.username.clone();
    let password = data.password.clone();
    let hashed_password = hash_password_to_string(password);
    match query_as::<_, ExistingUser>("SELECT UserId FROM users WHERE Username = ? AND Password = ?").bind(username).bind(hashed_password).fetch_one(&mut *db).await {
        Ok(value) => {
            cookies.add_private(Cookie::new("user_id", value.user_id.to_string()));
            Ok(Redirect::to("/"))
        },
        Err(error) => {
            Err(BadRequest(Some(error.to_string())))
        }
    }
}

#[post("/signup", data="<data>")]
async fn signup(data: Form<NewUser<'_>>, mut db: Connection<ThreadsDatabase>, cookies: &CookieJar<'_>) -> Result<Redirect, BadRequest<String>> {
    let username = data.username.clone();
    let password = data.password.clone();
    let hashed_password = hash_password_to_string(password);
    println!("{}", hashed_password);
    match query("INSERT INTO users (Username, Password) VALUES (?, ?)")
        .bind(username)
        .bind(hashed_password)
        .execute(&mut *db)
        .await 
    {
        Ok(value) => {
            println!("{:?}", value);
            match query_as::<_, ExistingUser>("SELECT UserId FROM users WHERE Username = ?").bind(username).fetch_one(&mut *db).await {
                Ok(user) => {
                    cookies.add_private(Cookie::new("user_id", user.user_id.to_string()));
                    Ok(Redirect::to("/"))
                },
                Err(error) => {
                    println!("select user id error");
                    Err(BadRequest(Some(error.to_string())))
                }
            }
        },
        Err(error) => {
            println!("{}", error);
            Err(BadRequest(Some(error.to_string())))
        }
    }
}

#[post("/signout")]
async fn signout(_user: &ExistingUser, cookies: &CookieJar<'_>) -> String {
    cookies.remove_private(Cookie::named("user_id"));
    format!("Successfully signed out")
}

#[get("/getthreads")]
async fn get_threads(mut db: Connection<ThreadsDatabase>, user: &ExistingUser) -> Result<Json<Vec<Thread>>, BadRequest<String>> {
    match query_as::<_, Thread>("SELECT Floss, Amount FROM threads WHERE UserId = ?").bind(user.user_id).fetch_all(&mut *db).await {
        Ok(value) => {
            Ok(Json(value))
        },
        Err(error) => {
            Err(BadRequest(Some(error.to_string())))
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum UpdateThreadOptions {
    Increment = 0,
    Decrement = 1
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct UpdateThreadMessage {
    floss: i32,
    action: UpdateThreadOptions
}

#[post("/updatethread", data = "<data>")]
async fn update_thread(mut db: Connection<ThreadsDatabase>, user: &ExistingUser, data: Json<UpdateThreadMessage>) -> Result<String, BadRequest<String>> {
    let operator = match data.action {
        UpdateThreadOptions::Increment => "+",
        UpdateThreadOptions::Decrement => "-"
    };
    let statement = format!("UPDATE threads SET Amount = Amount {} 1 WHERE UserId = ? AND Floss = ?", operator);
    match query_as::<_, Thread>("SELECT * FROM threads WHERE UserId = ? AND Floss = ?").bind(user.user_id).bind(data.floss).fetch_optional(&mut *db).await {
        Ok(value) => {
            match value {
                Some(thread) => {
                    match (thread.amount, data.action) {
                        (1, UpdateThreadOptions::Decrement) => {
                            match sqlx::query("DELETE FROM threads WHERE UserId = ? AND Floss = ?").bind(user.user_id).bind(data.floss).execute(&mut *db).await {
                                Ok(value) => {
                                    Ok(format!("{value:?}"))
                                },
                                Err(error) => {
                                    Err(BadRequest(Some(error.to_string())))
                                }
                            }
                        },
                        (_, _) => {
                            match sqlx::query(&statement).bind(user.user_id).bind(data.floss).execute(&mut *db).await {
                                Ok(value) => {
                                    Ok(format!("{value:?}"))
                                },
                                Err(error) => {
                                    Err(BadRequest(Some(error.to_string())))
                                }
                            }
                        },
                    }
                },
                None => {
                    println!("No thread");
                   match data.action {
                    UpdateThreadOptions::Increment => {
                        match sqlx::query("INSERT INTO threads (UserId, Floss, Amount) VALUES (?, ?, 1)").bind(user.user_id).bind(data.floss).execute(&mut *db).await {
                            Ok(value) => {
                                Ok(format!("{value:?}"))
                            },
                            Err(error) => {
                                Err(BadRequest(Some(error.to_string())))
                            }
                        }
                    },
                    UpdateThreadOptions::Decrement => {
                        Err(BadRequest(Some(String::from("Unable to decrement a non-existent thread"))))
                    } 
                   }
                }
            }
        },
        Err(error) => {
            Err(BadRequest(Some(error.to_string())))
        }
    }
}

//#[post("/updatethreads", data="<data>")]
//fn update_threads(mut db: Connection<ThreadsDatabase>, user: &ExistingUser, data: Json<Vec<UpdateThreadMessage>>) {
//    let sql: String = String::new();
//    for thread in data.iter() {
//        let operator = match thread.action {
//            UpdateThreadOptions::Increment => "+",
//            UpdateThreadOptions::Decrement => "-"
//        };
//        let statement = format!("UPDATE threads SET Amount = Amount {} 1 WHERE UserId = ? AND Floss = ?;", operator);
//    }
//}


#[get("/testhtml")]
async fn test_html() -> RawHtml<String> {
    println!("test html");
    RawHtml(String::from(include_str!("../test.html")))
}

#[get("/<file..>", rank = 3)]
async fn build_dir(file: PathBuf) -> io::Result<NamedFile> {
    println!("any file: {file:?}");
    NamedFile::open(Path::new("").join(file)).await
}


#[launch]
fn rocket() -> _ {
    let paths = fs::read_dir("../").unwrap();
    
    for path in paths {
        println!("Name: {}", path.unwrap().path().display())
    }

    rocket::build()
        .attach(ThreadsDatabase::init())
        .attach(Cors)
        .mount("/", routes![all_options, index, login_page, login, signup, signout, get_threads, test_html, build_dir, update_thread, login_redirect])
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