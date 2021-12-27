use std::os::unix::ffi::OsStrExt;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let verbose = true;
    if verbose { eprintln!("initializing..."); }
    let address = "127.0.0.1:8080";

    let server = actix_web::HttpServer::new(|| {
        actix_web::App::new()
            .route("/*", actix_web::web::get().to(git))
    });
    if verbose { eprintln!("binding to {}...", address); }
    let server = server.bind(address)?;
    if verbose { eprintln!("running..."); }
    let future = server.run();
    if verbose { eprintln!("ready to serve"); }
    future.await?;
    if verbose { eprintln!("exited gracefully"); }
    Ok(())
}

async fn git(request: actix_web::HttpRequest) -> impl actix_web::Responder {
    eprintln!("{} {} {}", request.method(), request.path(), request.peer_addr().unwrap());
    let mut body: Vec<u8> = Vec::new();

    body.extend_from_slice(request.path().as_bytes());
    body.extend_from_slice("\n".as_bytes());

    let paths = std::fs::read_dir(format!("./{}", request.path())).unwrap();
    for path in paths {
        body.extend_from_slice(path.unwrap().file_name().as_bytes());
        body.extend_from_slice("\n".as_bytes());
    }
    actix_web::HttpResponse::Ok().body(body)
}