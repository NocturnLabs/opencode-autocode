use actix_web::{web, App, HttpServer};
use anyhow::Result;

pub mod routes;
pub mod static_files;

/// Main entry point to run the web server
#[actix_web::main]
pub async fn run_server(port: u16, open_browser: bool) -> Result<()> {
    let addr = format!("127.0.0.1:{}", port);
    println!("🚀 Starting web dashboard at http://{}", addr);

    if open_browser {
        let url = format!("http://{}", addr);
        if let Err(e) = open::that(&url) {
            eprintln!("⚠️  Failed to open browser: {}", e);
        }
    }

    HttpServer::new(move || {
        App::new()
            .service(
                web::scope("/api")
                    .service(routes::get_status)
                    .service(routes::get_runs)
                    .service(routes::get_run_by_id)
                    .service(routes::run_project),
            )
            .default_service(web::to(static_files::index))
    })
    .bind(&addr)?
    .run()
    .await?;

    Ok(())
}
