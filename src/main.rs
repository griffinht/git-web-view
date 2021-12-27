#[actix_web::main]
async fn main() -> std::io::Result<()> {
    eprintln!("initializing...");
    let address = "127.0.0.1:8080";

    let server = actix_web::HttpServer::new(|| {
        actix_web::App::new()
            .route("/", actix_web::web::get().to(git))
    });
    eprintln!("binding to {}...", address);
    let server = server.bind(address)?;
    eprintln!("running...");
    let future = server.run();
    eprintln!("ready to serve");
    future.await?;
    eprintln!("exited gracefully");
    Ok(())
}

async fn git() -> impl actix_web::Responder {
    eprintln!("hello");
    actix_web::HttpResponse::Ok().body("Hello world!")
}