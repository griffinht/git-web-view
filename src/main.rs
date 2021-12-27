#[actix_web::main]
async fn main() -> std::io::Result<()> {
    eprintln!("starting git-web-view server");
    let address = "127.0.0.1:8080";

    let server = actix_web::HttpServer::new(|| {
        actix_web::App::new()
            .route("/", actix_web::web::get().to(git))
    });
    eprintln!("initialized");
    let server = server.bind(address)?;
    eprintln!("bound to {}", address);
    let future = server.run();
    eprintln!("running");
    future.await?;
    eprintln!("exited gracefully");
    Ok(())
}

async fn git() -> impl actix_web::Responder {
    eprintln!("hello");
    actix_web::HttpResponse::Ok().body("Hello world!")
}