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

    body.extend_from_slice(format!("<h1>{}</h1>", path).as_bytes());

    let paths = match std::fs::read_dir(path) {
        Ok(paths) => paths,
        Err(err) => { eprintln!("{}", err); return actix_web::HttpResponse::NotFound().finish(); }
    };
    for path in paths {
        body.extend_from_slice(format!("<a href=\"{0}\">{0}</a>\n", match path.unwrap().file_name().into_string() {
            Ok(string) => { string }
            Err(_) => { eprintln!("couldn't convert path from OsString to String"); return actix_web::HttpResponse::InternalServerError().finish(); }
        }).as_bytes());
    }
    actix_web::HttpResponse::Ok().content_type("text/html").body(body)
}


fn serve_file(path: &String) -> actix_web::HttpResponse {
    let mut body: Vec<u8> = Vec::new();
    body.extend_from_slice(format!("<h3>{0}</h3><pre>", path).as_bytes());
    let string = match std::fs::read_to_string(path) {
        Ok(f) => { f }
        Err(err) => { eprintln!("{}", err); return actix_web::HttpResponse::NotFound().finish(); }
    }.replace("&", "&amp").replace("<", "&lt").replace(">", "&gt");
    body.extend_from_slice(string.as_bytes());
    body.extend_from_slice("</pre>".as_bytes());
    actix_web::HttpResponse::Ok().body(body)
}