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

    // Register instance globally
    let pid = std::process::id();
    let instance_repo = crate::db::InstanceRepository::open()?;
    let instance_id = instance_repo.register(pid, "web", None)?;
    println!("Process registered as instance #{}", instance_id);

    // Initial heartbeat
    let instance_id_clone = instance_id;

    // Background thread for heartbeats
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(30));
            // Open new connection for heartbeat thread
            if let Ok(repo) = crate::db::InstanceRepository::open() {
                let _ = repo.heartbeat(instance_id_clone);
            }
        }
    });

    let server = HttpServer::new(move || {
        App::new()
            .service(
                web::scope("/api")
                    .service(routes::get_status)
                    .service(routes::get_runs)
                    .service(routes::get_run_by_id)
                    .service(routes::run_project)
                    .service(routes::get_instances)
                    .service(routes::get_instance_logs),
            )
            .default_service(web::to(static_files::index))
    })
    .bind(&addr)?
    .run();

    let result = server.await;

    // Cleanup
    let _ = instance_repo.mark_stopped(instance_id);

    result?;

    Ok(())
}
