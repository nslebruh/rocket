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

// define database struct (class)
#[derive(Database)] // auto implement the Database trait using a derive macro
#[database("mysql_test")] // specify name of database
pub struct ThreadsDatabase(sqlx::MySqlPool);


// define empty struct to deal with CORS (NOT MY CODE this is from StackOverflow)
pub struct Cors;

// implement an async trait using a async trait macro
// this allows my webserver to respond to CORS requests and OPTION requests
#[async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info {
            name: "Cross-Origin-Resource-Sharing Fairing",
            kind: Kind::Response,
        }
    }
    // this sets headers on all responses so that I do not have to deal with CORS
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

// define an existing user struct that can be serialized, deserialized, cloned and extracted from a html form POST request
#[derive(Deserialize, Serialize, Debug, FromForm, Clone)]
pub struct ExistingUser {
    user_id: i32, // contains a single number representing the existing user's ID
}

// allow an existing user to be created from a MySQL row returned from a database query
impl FromRow<'_, MySqlRow> for ExistingUser {
    fn from_row(row: &'_ MySqlRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            user_id: row.try_get("id")? // question mark at the end of this line allows the error to bubble up and will auto return an error if it has one
        })
    }
}

// implement async trait
#[async_trait]
// this allows an existing user to be retrieved using the contents of a request - type safe validation 
impl<'r> FromRequest<'r> for &'r ExistingUser {
    type Error = std::convert::Infallible;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let user_result: &Option<ExistingUser> = req.local_cache_async(async {
            // get a reference to the database struct
            // this will fail if it cannot get the database which is good because the entire project is useless without the database
            let db = req.guard::<&ThreadsDatabase>().await.succeeded().unwrap(); 
            // get a user ID from the request's secure http cookie 
            if let Some(value) = req.cookies().get_private("user_id") {
                // parse string into number
                if let Ok(id) = value.value().parse::<i32>() {
                    // query the database
                    if let Ok(res) = query("SELECT id FROM users WHERE id = ?").bind(id).execute(&**db).await {
                        println!("{:?}", res);
                        // return existing user if query is successful
                        return Some(ExistingUser { user_id: id })
                    }
                }   
            }
            // return None if any errors occur
            None
            // old code below
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
        // match user_result using pattern matching
        // if there is an existing user return a successful outcome with the existing user as the value
        // if there is not an existing user forward the request
        match user_result {
            Some(value) => Outcome::Success(value),
            None => Outcome::Forward(())
        }
    }
}

// define thread struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thread {
    pub floss: i32,
    pub amount: i32,
    pub name: String,
    pub color: String,
}

// implement FromRow to get a thread from a MySQL row
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

// define new user struct
#[derive(Deserialize, Serialize, Debug, FromForm, Copy, Clone)]
pub struct NewUser<'r> {
    username: &'r str,
    password: &'r str
}

// display stuff I can probably remove
impl <'r> Display for NewUser<'r> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Username: {}, Password: {}", self.username, self.password)
    }
}
// turn thread to json for sending in responses
impl <'r> From<NewUser<'r>> for String {
    fn from(value: NewUser) -> Self {
        format!("{{\"data\": \"username: {}, password: {}\"}}", value.username, value.password)
    }
}

// enum for the way to update a thread
#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum UpdateThreadOptions {
    Increment = 0,
    Decrement = 1
}

// struct that contains the update message sent when the increment or decrement button on the front end is clicked
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

// sends this as the 404 error message
#[catch(404)]
fn not_found(_: &Request) -> &'static str {
    "404 funny not found"
}

// sends this as the 500 error message
#[catch(500)]
fn oops(_: &Request) -> &'static str {
    "500: Oops all server error"
}

// main path that returns the main page
// this only responds to requests that return a ExistingUser using the FromRequest implementation above
// if the request does not meet the requirement it forwards it to the next path that matches
#[get("/")]
async fn index(_user: &ExistingUser) -> RawHtml<String> {
    RawHtml(include_str!("files/main_page.html").to_string())
}

// this path will be matched if a request to the main path does not meet the ExistingUser requirement
// redirects to the login path
// essentially if the user is not signed in when the go to the main path they will be redirected to the login page path
#[get("/", rank = 2)]
async fn login_redirect() -> Redirect {
    Redirect::to("/login")
}

// login path
// returns the login page
#[get("/login")]
async fn login_page() -> RawHtml<String> {
    RawHtml(include_str!("files/login.html").to_string())
}

// login POST path
// when a user logs in, the form POST request is sent to this
// returns a redirect to the main page if login is successful or redirect to the login page if unsuccessful
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

// signup POST path
// when a user signs up, the form POST request is sent here
// does the same thing as the login path but creates a user instead of checking if the user already exists
// returns the same thing
#[post("/signup", data="<data>")]
async fn signup(data: Form<NewUser<'_>>, mut db: Connection<ThreadsDatabase>, cookies: &CookieJar<'_>) -> Redirect {
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
                    Redirect::to("/")
                },
                Err(error) => {
                    println!("select user id error: {}", error.to_string());
                    Redirect::to("/login")
                }
            }
        },
        Err(error) => {
            println!("signup error: {}", error.to_string());
            Redirect::to("/login")
        }
    }
}

// signout POST path
// signs the user out
// the user already needs to be signed in to sign out
// probably should return a redirect to the login page but I don't care at this point this is fine
#[post("/signout")]
async fn signout_post(_user: &ExistingUser, cookies: &CookieJar<'_>) -> String {
    cookies.remove_private(Cookie::named("user_id"));
    format!("Successfully signed out")
}

// signout GET path
// also signs the user out
// does the same as the POST path but this makes it easier for me to implement on the frontend
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

// path that accepts a vector of threads and does a mass update in the database to match
// frontend does a POST request on closing that includes a vector of all threads that have a non-zero amount
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

// path that returns a test page that allows me to test most of the api's endpoints (paths)
#[get("/testhtml")]
async fn test_html() -> RawHtml<String> {
    println!("test html");
    RawHtml(String::from(include_str!("files/test.html")))
}

// Favicon struct so that I can send the website's icon without figuring out how I am actually meant to
// It literally only contains the bytes of ../favicon.ico
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default, Responder)]
#[response(status = 200, content_type = "image/x-icon")]
struct Favicon(&'static [u8]);

// sends the website icon (favicon)
#[get("/favicon.ico")]
async fn favicon() -> Favicon {
    Favicon(include_bytes!("files/favicon.ico"))
}

// main function that runs the webserver
#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(ThreadsDatabase::init())
        .attach(Cors)
        .mount("/", routes![all_options, index, login_page, login, signup, get_threads, test_html,  update_thread, login_redirect, favicon, signout_get, signout_post, update_threads])
        .register("/", catchers![not_found, oops])
}

// password hashing function
// returns the salted and hashed password as an owned string
fn hash_password_to_string(password: &str) -> String {
    let salt = format!("{password}_salt_lol"); // salt
    let mut output_password = [0u8; 32]; // initialise output password in memory as 32 bytes
    Argon2::default().hash_password_into(
      password.as_bytes(),
      salt.as_bytes(),
      &mut output_password
    ).expect("unable to hash password");
    encode(output_password)
}

#[cfg(test)]
mod tests {
    use super::rocket;
    use rocket::local::blocking::Client;
    use rocket::http::Status;

    #[test]
    fn testing_page() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get(uri!(super::test_html)).dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string().unwrap(), include_str!("files/test.html"));
    }
}

