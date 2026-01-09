use actix_web::{body::BoxBody, HttpRequest, HttpResponse, Responder};
use mime_guess::from_path;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "src/web/frontend/dist/"]
struct Asset;

pub async fn index(req: HttpRequest) -> impl Responder {
    let path = if req.path() == "/" {
        "index.html"
    } else {
        req.path().trim_start_matches('/')
    };

    match Asset::get(path) {
        Some(content) => {
            let mime = from_path(path).first_or_octet_stream();
            HttpResponse::Ok()
                .content_type(mime.as_ref())
                .body(BoxBody::new(content.data.into_owned()))
        }
        None => {
            // Fallback to index.html for SPA-like behavior or if file not found
            match Asset::get("index.html") {
                Some(content) => HttpResponse::Ok()
                    .content_type("text/html")
                    .body(BoxBody::new(content.data.into_owned())),
                None => HttpResponse::NotFound().body("404 Not Found"),
            }
        }
    }
}
