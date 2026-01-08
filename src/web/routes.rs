use crate::db;
use crate::db::sessions::{Session, SessionEvent};
use actix_web::{get, post, web, HttpResponse, Responder};
use serde::Serialize;

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
    // This is a simplified version. In a real app, you'd trigger the vibe loop in a background thread.
    // For now, we'll just acknowledge the request.
    println!("Web Trigger: Starting vibe loop (simulated)");
    HttpResponse::Ok().body("Vibe loop triggered!")
}
