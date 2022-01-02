fn serve_static(static_directory: &Option<String>, path: &str, request_path: &str) -> actix_web::HttpResponse {
    // serve from static directory
    match static_directory {
        None => {}
        Some(static_directory) => {
            match std::fs::read(format!("{}{}", static_directory, path)) {
                Ok(file) => {
                    return actix_web::HttpResponse::Ok().body(file)
                }
                Err(error) => {
                    if error.kind() != std::io::ErrorKind::NotFound {
                        eprintln!("error reading {} while serving static directory {}: {}", path, static_directory, error);
                        return actix_web::HttpResponse::InternalServerError().finish();
                    }
                    // otherwise ignore and try serving from files.rs
                }
            }
        }
    }
    // serve from files.rs
    for file in crate::files::STATICS {
        if file.path.eq(request_path) {
            let extension = match std::path::Path::new(path).extension().and_then(std::ffi::OsStr::to_str) {
                None => { eprintln!("invalid path {}", request_path); return actix_web::HttpResponse::InternalServerError().finish(); },
                Some(extension) => extension
            };

            return actix_web::HttpResponse::Ok().content_type(actix_files::file_extension_to_mime(&extension).to_string()).body(file.contents);
        }
    }
    return actix_web::HttpResponse::NotFound().finish();
}

fn serve_template(templates: &std::collections::HashMap<String, Vec<crate::template::Parsed>>, template_name: &str, path: &str, request_path: &str) -> actix_web::HttpResponse {
    let template = match templates.get(template_name) {
        None => { eprintln!("no template for {}", template_name); return actix_web::HttpResponse::InternalServerError().finish(); }
        Some(template) => template
    };
    let mut body: Vec<u8> = Vec::new();
    for parsed in template {
        body.extend_from_slice(&parsed.buf);
        match &parsed.tag {
            None => { }
            Some(tag) => {
                match tag.as_str() {
                    "NAV" => {
                        body.extend_from_slice(&crate::template::nav::get_nav(request_path));
                    }
                    "DIRECTORY" => { match &crate::template::links::get_links(&path) {
                        Ok(links) => { body.extend_from_slice(links); }
                        Err(err) => { eprintln!("error getting links from {}: {}", path, err); return actix_web::HttpResponse::InternalServerError().finish(); }
                    } }
                    "PATH" => { body.extend_from_slice(request_path.as_bytes()); }
                    "FILE" => {
                        let string = match std::fs::read_to_string(path) {
                            Ok(f) => { f }
                            Err(err) => { eprintln!("error reading file to string: {}", err); return actix_web::HttpResponse::NotFound().finish(); }
                        };
                        if request_path.ends_with(".md") {
                            let options = pulldown_cmark::Options::empty();
                            let parser = pulldown_cmark::Parser::new_ext(&string, options);
                            let mut output_string = String::new();
                            pulldown_cmark::html::push_html(&mut output_string, parser);
                            body.extend(output_string.as_bytes());
                        } else {//todo read with bufread?
                            body.extend("<pre>".as_bytes());
                            let string = string
                                .replace("&", "&amp")//todo each replace is very slow
                                .replace("<", "&lt")
                                .replace(">", "&gt");
                            body.extend(string.as_bytes());
                            body.extend("</pre>".as_bytes());
                        }
                    }
                    _ => {
                        if tag.starts_with("LINK") {
                            fn get_repeated_string(string: &str, i: i32) -> String {
                                let mut dots: Vec<&str> = Vec::new();
                                for _ in 0..i {
                                    dots.push(string);
                                }
                                return dots.join("");
                            }
                            body.extend_from_slice(format!("{}{}", get_repeated_string("../", request_path.matches("/").count() as i32 - 1), &tag[5..]).as_bytes());
                        } else {
                            eprintln!("unknown tag {}", tag); return actix_web::HttpResponse::InternalServerError().finish();
                        }
                    }
                }
            }
        };
    }
    return actix_web::HttpResponse::Ok().content_type("text/html").body(body);
}

pub async fn response(request: actix_web::HttpRequest, state: actix_web::web::Data<crate::State>) -> impl actix_web::Responder {
    eprintln!("{} {} {}", request.peer_addr().unwrap(), request.method(), request.path());

    //todo prevent filesystem traversal with ../../.. or something
    let path = match &state.directory {
        None => { format!(".{}", request.path()) }
        Some(directory) => { format!("{}{}", directory, request.path()) }
    };

    match git2::Repository::open(&path) {
        Ok(repository) => { eprintln!("{}", repository.head().unwrap().name().unwrap()); eprintln!("{}", repository.path().display()); },
        Err(error) => { eprintln!("error opening git repository: {}", error); }
    };

    let metadata = match std::fs::metadata(&path) {
        Ok(file) => { file }
        Err(err) => {
            if err.kind() != std::io::ErrorKind::NotFound {
                eprintln!("{}", err); return actix_web::HttpResponse::NotFound().finish();
            }
            return serve_static(&state.static_directory, &path, request.path());
        }
    };
    //todo symlink support?

    let template_name;
    if metadata.is_dir() { template_name = "/directory.html"; }
    else if metadata.is_file() { template_name = "/file.html"; }
    else { eprintln!("not a file or a directory"); return actix_web::HttpResponse::NotFound().finish(); }

    serve_template(&state.templates, template_name, &path, request.path())
}