use std::{collections::HashMap, io::Write, sync::Mutex};

use actix_cors::Cors;
use actix_web::{http::header, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use async_trait::async_trait;
#[allow(unused_imports)]
use reqwest::Client as HttpClient;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Task {
    id: u64,
    name: String,
    completed: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct User {
    id: u64,
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Database {
    tasks: HashMap<u64, Task>,
    users: HashMap<u64, User>,
}

impl Database {
    fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            users: HashMap::new(),
        }
    }

    // CRUD Data
    fn insert_or_update_task(&mut self, task: Task) {
        self.tasks.insert(task.id, task);
    }

    fn get_task(&self, id: &u64) -> Option<&Task> {
        self.tasks.get(id)
    }

    fn get_all_tasks(&self) -> Vec<&Task> {
        self.tasks.values().collect()
    }

    fn delete_task(&mut self, id: &u64) -> Option<Task> {
        self.tasks.remove(id)
    }

    fn insert_or_update_user(&mut self, user: User) {
        self.users.insert(user.id, user);
    }

    #[allow(dead_code)]
    fn get_user(&self, id: &u64) -> Option<&User> {
        self.users.get(id)
    }

    fn get_user_by_name(&self, username: &str) -> Option<&User> {
        self.users.values().find(|u| u.username == username)
    }

    #[allow(dead_code)]
    fn get_all_users(&self) -> Vec<&User> {
        self.users.values().collect()
    }

    #[allow(dead_code)]
    fn delete_user(&mut self, id: &u64) -> Option<User> {
        self.users.remove(id)
    }

    // SAVE TO FILE
    fn save_to_file(&self) -> std::io::Result<()> {
        let data = serde_json::to_string(&self)?;
        let mut file = std::fs::File::create("database.json")?;
        file.write_all(data.as_bytes())?;
        Ok(())
    }

    fn load_from_file() -> std::io::Result<Self> {
        let file_content = std::fs::read_to_string("database.json")?;
        let db = serde_json::from_str(&file_content)?;
        Ok(db)
    }
}

struct AppState {
    database: Mutex<Database>,
}

async fn create_task(app_state: web::Data<AppState>, task: web::Json<Task>) -> impl Responder {
    let mut db = app_state.database.lock().unwrap();
    db.insert_or_update_task(task.into_inner());
    let _ = db.save_to_file();
    HttpResponse::Created().finish()
}

async fn read_task(app_state: web::Data<AppState>, id: web::Path<u64>) -> impl Responder {
    let db = app_state.database.lock().unwrap();
    match db.get_task(&id.into_inner()) {
        Some(task) => HttpResponse::Ok().json(task),
        None => HttpResponse::NotFound().finish(),
    }
}

async fn update_task(app_state: web::Data<AppState>, task: web::Json<Task>) -> impl Responder {
    let mut db = app_state.database.lock().unwrap();
    db.insert_or_update_task(task.into_inner());
    let _ = db.save_to_file();
    HttpResponse::Created().finish()
}

async fn delete_task(app_state: web::Data<AppState>, id: web::Path<u64>) -> impl Responder {
    let mut db = app_state.database.lock().unwrap();
    HttpResponse::Ok().json(&db.delete_task(&id))
}

async fn read_all_tasks(app_state: web::Data<AppState>) -> impl Responder {
    let db = app_state.database.lock().unwrap();
    let all_tasks = db.get_all_tasks();
    HttpResponse::Ok().json(all_tasks)
}

async fn register_user(app_state: web::Data<AppState>, user: web::Json<User>) -> impl Responder {
    let mut db = app_state.database.lock().unwrap();
    db.insert_or_update_user(user.into_inner());
    let _ = db.save_to_file();
    HttpResponse::Created().finish()
}

async fn login(app_state: web::Data<AppState>, user: web::Json<User>) -> impl Responder {
    let db = app_state.database.lock().unwrap();
    match db.get_user_by_name(&user.username) {
        Some(stored_user) if stored_user.password == user.password => {
            HttpResponse::Ok().body("Logged in!")
        }
        _ => HttpResponse::BadRequest().body("Invalid username or password"),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let database = match Database::load_from_file() {
        Ok(db) => db,
        Err(_) => Database::new(),
    };
    let data = web::Data::new(AppState {
        database: Mutex::new(database),
    });

    // move moves everything from main() scope into the following closure
    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::permissive()
                    .allowed_origin_fn(|origin, _req_head| {
                        origin.as_bytes().starts_with(b"http://localhost") || origin == "null"
                    })
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                    .allowed_headers(vec![
                        header::AUTHORIZATION,
                        header::ACCEPT,
                        header::CONTENT_TYPE,
                    ])
                    .supports_credentials()
                    .max_age(3600),
            )
            .app_data(data.clone()) // just cloning the Mutex smart pointer
            .route("/task", web::post().to(create_task))
            .route("/task/{id}", web::get().to(read_task))
            .route("/task/{id}", web::put().to(update_task))
            .route("task", web::get().to(read_all_tasks))
            .route("task/{id}", web::delete().to(delete_task))
            .route("/register", web::post().to(register_user))
            .route("/login", web::post().to(login))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
