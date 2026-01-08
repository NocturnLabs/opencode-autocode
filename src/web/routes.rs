use crate::db;
use crate::db::sessions::{Session, SessionEvent};
use actix_web::{get, post, web, HttpResponse, Responder};
use serde::Serialize;
use std::fs::File;
use std::io::Read;

#[derive(Serialize)]
struct StatusResponse {
    status: String,
    project_name: String,
    features_passing: i32,
    features_remaining: i32,
}

#[derive(Serialize)]
struct RunDetailResponse {
    session: Session,
    events: Vec<SessionEvent>,
}

#[get("/status")]
pub async fn get_status() -> impl Responder {
    // Try to get stats from database
    let db_path = std::path::PathBuf::from(db::DEFAULT_DB_PATH);
    if !db_path.exists() {
        return HttpResponse::Ok().json(StatusResponse {
            status: "No database found".to_string(),
            project_name: "Unknown".to_string(),
            features_passing: 0,
            features_remaining: 0,
        });
    }

    match db::Database::open(&db_path) {
        Ok(database) => {
            let (passing, remaining) = database.features().count().unwrap_or((0, 0));
            HttpResponse::Ok().json(StatusResponse {
                status: "active".to_string(),
                project_name: "Current Project".to_string(),
                features_passing: passing as i32,
                features_remaining: remaining as i32,
            })
        }
        Err(_) => HttpResponse::InternalServerError().body("Failed to open database"),
    }
}

#[get("/runs")]
pub async fn get_runs() -> impl Responder {
    let db_path = std::path::PathBuf::from(db::DEFAULT_DB_PATH);
    if !db_path.exists() {
        return HttpResponse::Ok().json(Vec::<Session>::new());
    }

    match db::Database::open(&db_path) {
        Ok(database) => match database.sessions().list_sessions() {
            Ok(sessions) => HttpResponse::Ok().json(sessions),
            Err(_) => HttpResponse::InternalServerError().body("Failed to list sessions"),
        },
        Err(_) => HttpResponse::InternalServerError().body("Failed to open database"),
    }
}

#[get("/runs/{id}")]
pub async fn get_run_by_id(path: web::Path<i64>) -> impl Responder {
    let session_id = path.into_inner();
    let db_path = std::path::PathBuf::from(db::DEFAULT_DB_PATH);
    if !db_path.exists() {
        return HttpResponse::NotFound().body("Database not found");
    }

    match db::Database::open(&db_path) {
        Ok(database) => match database.sessions().get_session_with_events(session_id) {
            Ok(Some((session, events))) => {
                HttpResponse::Ok().json(RunDetailResponse { session, events })
            }
            Ok(None) => HttpResponse::NotFound().body("Session not found"),
            Err(_) => HttpResponse::InternalServerError().body("Failed to get session"),
        },
        Err(_) => HttpResponse::InternalServerError().body("Failed to open database"),
    }
}

#[post("/run")]
pub async fn run_project() -> impl Responder {
    println!("Web Trigger: Starting vibe loop (simulated)");
    HttpResponse::Ok().body("Vibe loop triggered!")
}

#[get("/instances")]
pub async fn get_instances() -> impl Responder {
    match crate::db::InstanceRepository::open() {
        Ok(repo) => match repo.list(false) {
            Ok(instances) => HttpResponse::Ok().json(instances),
            Err(_) => HttpResponse::InternalServerError().body("Failed to list instances"),
        },
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Failed to open registry: {}", e))
        }
    }
}

#[get("/instances/{id}/logs")]
pub async fn get_instance_logs(path: web::Path<i64>) -> impl Responder {
    let instance_id = path.into_inner();

    let repo = match crate::db::InstanceRepository::open() {
        Ok(r) => r,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to open registry: {}", e))
        }
    };

    match repo.get(instance_id) {
        Ok(Some(instance)) => {
            if let Some(log_path) = instance.log_path {
                let path = std::path::Path::new(&log_path);
                if path.exists() {
                    if let Ok(mut file) = File::open(path) {
                        let mut contents = String::new();
                        if file.read_to_string(&mut contents).is_ok() {
                            return HttpResponse::Ok().content_type("text/plain").body(contents);
                        }
                    }
                }
                HttpResponse::NotFound().body("Log file not found")
            } else {
                HttpResponse::NotFound().body("No log path for this instance")
            }
        }
        Ok(None) => HttpResponse::NotFound().body("Instance not found"),
        Err(_) => HttpResponse::InternalServerError().body("Failed to get instance"),
    }
}
