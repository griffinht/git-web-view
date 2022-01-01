use std::fs::File;
use std::io::{BufReader, Read};

pub async fn response(request: actix_web::HttpRequest, state: actix_web::web::Data<std::collections::HashMap<String, Vec<crate::template::Parsed>>>) -> impl actix_web::Responder {
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
    let template = match state.get(template_name) {
        None => { eprintln!("no template for {}", template_name); return actix_web::HttpResponse::InternalServerError().finish(); }
        Some(template) => template
    };

    /*let mut body: Vec<u8> = Vec::new();
    for parsed in template {
        body.extend_from_slice(&*parsed.buf);
        match &parsed.tag {
            None => { }
            Some(tag) => {
                match tag.as_str() {
                    "NAV" => {
                        body.extend_from_slice(&crate::template::nav::get_nav(request.path()));
                    }
                    "DIRECTORY" => { body.extend_from_slice(&crate::template::links::get_links(&path).unwrap()); }
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
    return actix_web::HttpResponse::Ok().content_type("text/html").body(body);*/
    struct Response  {
        template: std::collections::VecDeque<crate::template::Parsed>,
        request: actix_web::HttpRequest,
        reader: Option<std::io::BufReader<std::fs::File>>,
    }
    impl futures::Stream for Response {
        type Item = Result<actix_web::web::Bytes, std::io::Error>;

        fn poll_next(mut self: std::pin::Pin<&mut Self>, _: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
            let parsed = match self.template.pop_front() {
                None => { return std::task::Poll::Ready(None); }
                Some(parsed) => parsed
            };
            match &self.reader {
                None => {}
                Some(mut reader) => {
                    let mut body: Vec<u8> = Vec::new();
                    let n = reader.read(&mut body);
                    match n {
                        Ok(n) => { eprintln!("{}", n); }
                        Err(err) => { eprintln!("{}", err); }
                    }
                    self.reader = None;
                }
            }

            let path = format!("./{}", self.request.path());
            let mut body = Vec::from(parsed.buf);
            match parsed.tag {
                None => { }
                Some(tag) => {
                    match tag.as_str() {
                        "NAV" => {
                            body.extend_from_slice(&crate::template::nav::get_nav(self.request.path()));
                        }
                        "DIRECTORY" => { body.extend_from_slice(&crate::template::links::get_links(&path).unwrap()); }
                        "PATH" => { body.extend_from_slice(self.request.path().as_bytes()); }
                        "FILE" => {
                            let escape_html = true;
                            if escape_html {
                                body.extend(match std::fs::read_to_string(&path) {
                                    Ok(f) => { f }
                                    Err(err) => { eprintln!("error reading file to string: {}", err);
                                        return std::task::Poll::Ready(Some(Err(std::io::Error::new(std::io::ErrorKind::AddrInUse, "ssdfsdf"))));
                                        //return actix_web::HttpResponse::NotFound().finish();
                                    }
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
                                        return std::task::Poll::Ready(Some(Err(std::io::Error::new(std::io::ErrorKind::AddrInUse, "ssdfsdf"))));
                                        //return actix_web::HttpResponse::NotFound().finish();
                                    }
                                })
                            }
                        }
                        _ => { eprintln!("unknown tag {}", tag);
                            return std::task::Poll::Ready(Some(Err(std::io::Error::new(std::io::ErrorKind::AddrInUse, "ssdfsdf"))));
                            //return actix_web::HttpResponse::InternalServerError().finish();
                        }
                    }
                }
            };
            return std::task::Poll::Ready(Some(Ok(actix_web::web::Bytes::from(body))));
        }
    }

    return actix_web::HttpResponse::Ok().content_type("text/html").streaming(Response { template: std::collections::VecDeque::from(template.clone()), request, reader: None });
}
