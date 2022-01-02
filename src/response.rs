pub async fn response(request: actix_web::HttpRequest, state: actix_web::web::Data<crate::State>) -> impl actix_web::Responder {
    eprintln!("{} {} {}", request.peer_addr().unwrap(), request.method(), request.path());
    //todo prevent filesystem traversal with ../../.. or something
    let path = format!("{}{}", state.directory, request.path());
    let metadata = match std::fs::metadata(&path) {
        Ok(file) => { file }
        Err(err) => { eprintln!("{}", err); return actix_web::HttpResponse::NotFound().finish(); }
    };
    //todo symlink support?

    let template_name;
    if metadata.is_dir() { template_name = "directory.html"; }
    else if metadata.is_file() { template_name = "file.html"; }
    else { eprintln!("not a file or a directory"); return actix_web::HttpResponse::NotFound().finish(); }
    let template = match state.template.get(template_name) {
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
                        body.extend_from_slice(&crate::template::nav::get_nav(request.path()));
                    }
                    "DIRECTORY" => { body.extend_from_slice(&crate::template::links::get_links(&path).unwrap()); }
                    "PATH" => { body.extend_from_slice(request.path().as_bytes()); }
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
