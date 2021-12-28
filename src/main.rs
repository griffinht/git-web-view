use std::ops::Deref;
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
    //todo prevent filesystem traversal with ../../.. or something
    let path = format!("./{}", request.path());
    let metadata = match std::fs::metadata(&path) {
        Ok(file) => { file }
        Err(err) => { eprintln!("{}", err); return actix_web::HttpResponse::NotFound().finish(); }
    };
    //todo symlink support?
    if metadata.is_dir() { serve_directory(&path) }
    else if metadata.is_file() { serve_file(&path) }
    else { return actix_web::HttpResponse::Forbidden().finish(); }

}

fn serve_directory(path: &String) -> actix_web::HttpResponse {
    let mut body: Vec<u8> = Vec::new();

    body.extend_from_slice(path.as_bytes());
    body.extend_from_slice("\n".as_bytes());

    let paths = match std::fs::read_dir(path) {
        Ok(paths) => paths,
        Err(err) => { eprintln!("{}", err); return actix_web::HttpResponse::NotFound().finish(); }
    };
    for path in paths {
        body.extend_from_slice(path.unwrap().file_name().as_bytes());
        body.extend_from_slice("\n".as_bytes());
    }
    actix_web::HttpResponse::Ok().body(body)
}


fn serve_file(path: &String) -> actix_web::HttpResponse {
    let mut body: Vec<u8> = Vec::new();
    body.extend_from_slice(path.as_bytes());
    body.extend_from_slice("\n".as_bytes());
    body.extend_from_slice(&match std::fs::read(path) {
        Ok(f) => { f }
        Err(err) => { eprintln!("{}", err); return actix_web::HttpResponse::NotFound().finish(); }
    });
    actix_web::HttpResponse::Ok().body(body)
}