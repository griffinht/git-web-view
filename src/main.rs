mod options;
mod template;

#[macro_export]
macro_rules! default_bind_address {
    () => ("127.0.0.1:8080".to_string())
}

struct State {
    template: std::collections::HashMap<String, Vec<template::Parsed>>
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let matches = match options::matches(std::env::args().collect())? {
        None => { return Ok(()) }
        Some(matches) => { matches }
    };

    let verbose = !matches.opt_present("quiet");

    let template_path = "./default-template/";
    if verbose { eprintln!("parsing default template {}", template_path); }
    //todo
    if verbose { eprintln!("initializing..."); }

    let address = if matches.opt_present("bind") {
        matches.opt_get::<String>("bind").unwrap().unwrap()
    } else {
        default_bind_address!()
    };

    let server = actix_web::HttpServer::new(|| {
        actix_web::App::new()
            .data(State { template: template::parse_directory(template_path).unwrap() })
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

async fn git(request: actix_web::HttpRequest, state: actix_web::web::Data<State>) -> impl actix_web::Responder {
    eprintln!("{} {} {}", request.peer_addr().unwrap(), request.method(), request.path());
    //todo prevent filesystem traversal with ../../.. or something
    let path = format!("./{}", request.path());
    let metadata = match std::fs::metadata(&path) {
        Ok(file) => { file }
        Err(err) => { eprintln!("{}", err); return actix_web::HttpResponse::NotFound().finish(); }
    };
    //todo symlink support?
    eprintln!("{}", state.template.len());
    let template_name;
    if metadata.is_dir() { template_name = "directory.html"; }
    else if metadata.is_file() { template_name = "file.html"; }
    else { eprintln!("not a file or a directory"); return actix_web::HttpResponse::NotFound().finish(); }
    let template = state.template.get(template_name);

    if template.is_none() { eprintln!("no template for {}", template_name); return actix_web::HttpResponse::InternalServerError().finish(); }
    let template = template.unwrap();
    let mut body: Vec<u8> = Vec::new();
    for parsed in template {
        body.extend_from_slice(&*parsed.buf);
        match &parsed.tag {
            None => { }
            Some(tag) => {
                match tag.as_str() {
                    "NAV" => {
                        body.extend_from_slice(&get_nav(request.path()));
                    }
                    "DIRECTORY" => { body.extend_from_slice(&get_links(&path).unwrap()); }
                    "PATH" => { body.extend_from_slice(path.as_bytes()); }
                    "FILE" => {
                        let escape_html = true;
                        if escape_html {
                            body.extend(match std::fs::read_to_string(&path) {
                                Ok(f) => { f }
                                Err(err) => { eprintln!("error reading file to string: {}", err); return actix_web::HttpResponse::NotFound().finish(); }
                            }
                                .replace("&", "&amp")//todo each replace is very slow
                                .replace("<", "&lt")
                                .replace(">", "&gt")
                                .as_bytes());
                        } else {
                            body.extend(&match std::fs::read(&path) {
                                Ok(f) => { f }
                                Err(err) => {
                                    eprintln!("error reading file: {}", err);
                                    return actix_web::HttpResponse::NotFound().finish();
                                }
                            })
                        }
                    }
                    _ => { eprintln!("unknown tag {}", tag); return actix_web::HttpResponse::InternalServerError().finish(); }
                }
            }
        };
    }
    return actix_web::HttpResponse::Ok().content_type("text/html").body(body);
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
    fn get_directory_link(directory: &str, i: i32, trailing: &str) -> String {
        if i == 0 { // trailing (current) file/directory should not be a link
            format!("{}{}", directory, trailing)
        } else {
            format!("<a href=\"{}\">{}{}</a>", get_repeated_string("../", i), directory, trailing)
        }
    }
    for directory in request_path.rsplit("/") {
        if directory.is_empty() { continue; }
        // /example/dir/ <- needs to be added back for directories
        nav.push(get_directory_link(directory, i, "/"));
        i = i + 1;
    }
    // add leading slash -> /example/dir/
    nav.push(get_directory_link("/", i, ""));
    nav.reverse();
    let nav = nav;
    let mut real_nav: Vec<u8> = Vec::new();
    real_nav.extend_from_slice("<nav>".as_bytes());
    for n in nav {
        real_nav.extend_from_slice(n.as_bytes());
    }
    real_nav.extend_from_slice("</nav>".as_bytes());
    return real_nav;
}
fn get_links(path: &String) -> std::io::Result<Vec<u8>> {
    let mut links: Vec<u8> = Vec::new();
    let paths = std::fs::read_dir(path)?;
    for path in paths {
        let path = path.unwrap();
        links.extend(format!("<p><a href=\"{0}{1}\">{0}{1}</a></p>\n", match path.file_name().into_string() {
            Ok(string) => { string }
            Err(_) => { return Err(std::io::Error::new(std::io::ErrorKind::Other, "couldn't convert path from OsString to String")); }
        }, if path.file_type().unwrap().is_dir() { "/" } else { "" }).as_bytes());
    }
    return Ok(links);
}
