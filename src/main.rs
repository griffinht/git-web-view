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

    let server = actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .data(State { template: if matches.opt_present("template-directory") {
                template::parse_directory(template_path)
            } else {
                template::parse_directory_default()
            }
            })
            .route("/*", actix_web::web::get().to(git))
            .wrap(actix_web::middleware::Compress::new(
                if matches.opt_present("disable-compression") {
                    actix_web::http::ContentEncoding::Identity
                } else {
                    actix_web::http::ContentEncoding::Auto
                }))
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
                        body.extend_from_slice(&template::nav::get_nav(request.path()));
                    }
                    "DIRECTORY" => { body.extend_from_slice(&template::links::get_links(&path).unwrap()); }
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
