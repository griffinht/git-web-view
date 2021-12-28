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
    if metadata.is_dir() { serve_directory(&path, request.path()) }
    else if metadata.is_file() { serve_file(&path, request.path()) }
    else { return actix_web::HttpResponse::Forbidden().finish(); }

}

fn get_nav(request_path: &str) -> Vec<u8> {
    let mut nav: Vec<String> = Vec::new();
    let mut i = 0;
    fn get_repeated_string(string: &str, i: i32) -> String {
        let mut dots: Vec<&str> = Vec::new();
        for _ in 0..i {
            dots.push(string);
        }
        return dots.join("");
    }
    fn get_directory_link(directory: &str, i: i32) -> String {
        if i == 0 { // trailing (current) file/directory should not be a link
            String::from(directory)
        } else {
            format!("<a href=\"{0}\">{1}</a>", get_repeated_string("../", i), directory)
        }
    }
    let add_trailing_slash = request_path.ends_with("/");
    for directory in request_path.rsplit("/") {
        if directory.is_empty() { continue; }
        // /example/dir/ <- needs to be added back for directories
        let trailing_slash = if add_trailing_slash { "/" } else { "" };
        nav.push(format!("{}{}", get_directory_link(directory, i), trailing_slash));
        i = i + 1;
    }
    // add leading slash -> /example/dir/
    nav.push(get_directory_link("/", i));
    nav.reverse();
    let nav = nav;
    let mut real_nav: Vec<u8> = Vec::new();
    real_nav.extend_from_slice("<nav style=\"display: flex\">".as_bytes());
    for n in nav {
        real_nav.extend_from_slice(n.as_bytes());
    }
    real_nav.extend_from_slice("</nav>".as_bytes());
    return real_nav;
}

fn serve_directory(path: &String, request_path: &str) -> actix_web::HttpResponse {
    if !request_path.ends_with("/") { return actix_web::HttpResponse::TemporaryRedirect().header("location", format!("{}/", request_path)).finish(); }
    let mut body: Vec<u8> = Vec::new();

    body.extend(get_nav(request_path));

    //body.extend_from_slice("<p><a href=\".\">.</a></p>".as_bytes());
    body.extend_from_slice("<p><a href=\"..\">..</a></p>".as_bytes());
    let paths = match std::fs::read_dir(path) {
        Ok(paths) => paths,
        Err(err) => { eprintln!("{}", err); return actix_web::HttpResponse::NotFound().finish(); }
    };
    for path in paths {
        let path = path.unwrap();
        body.extend_from_slice(format!("<p><a href=\"{0}{1}\">{0}{1}</a></p>\n", match path.file_name().into_string() {
            Ok(string) => { string }
            Err(_) => { eprintln!("couldn't convert path from OsString to String"); return actix_web::HttpResponse::InternalServerError().finish(); }
        }, if path.file_type().unwrap().is_dir() { "/" } else { "" }).as_bytes());
    }
    actix_web::HttpResponse::Ok().content_type("text/html").body(body)
}

fn serve_file(path: &String, request_path: &str) -> actix_web::HttpResponse {
    let mut body: Vec<u8> = Vec::new();
    body.extend(get_nav(request_path));
    let start = std::time::Instant::now();
    let escape_html = true;
    if escape_html {
        body.extend_from_slice(match std::fs::read_to_string(path) {
            Ok(f) => { f }
            Err(err) => { eprintln!("{}", err); return actix_web::HttpResponse::NotFound().finish(); }
        }
            .replace("&", "&amp")//todo each replace is very slow
            .replace("<", "&lt")
            .replace(">", "&gt")
            .as_bytes());
    } else {
        body.extend_from_slice(&match std::fs::read(path) {
            Ok(f) => { f }
            Err(err) => {
                eprintln!("{}", err);
                return actix_web::HttpResponse::NotFound().finish();
            }
        })
    }
    eprintln!("{}microseconds", std::time::Instant::now().checked_duration_since(start).unwrap().as_micros());
    body.extend_from_slice("</pre>".as_bytes());
    actix_web::HttpResponse::Ok().content_type("text/html").body(body)
}