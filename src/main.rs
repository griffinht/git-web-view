mod options;
mod template;
mod git;

#[macro_export]
macro_rules! default_bind_address {
    () => ("127.0.0.1:8080".to_string())
}

pub struct State {
    template: std::collections::HashMap<String, Vec<template::Parsed>>
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let matches = match options::matches(std::env::args().collect())? {
        None => { return Ok(()) }
        Some(matches) => { matches }
    };

    let verbose = !matches.opt_present("quiet");

    if verbose { eprintln!("initializing..."); }

    let address = if matches.opt_present("bind") {
        matches.opt_get::<String>("bind").unwrap().unwrap()
    } else {
        default_bind_address!()
    };

    let server = actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .data(State { template: if matches.opt_present("template-directory") { //todo only do this once
                template::parse_directory(matches.opt_get::<String>("template-directory").unwrap().unwrap().as_str())
            } else {
                template::parse_directory_default()
            }
            })
            .route("/*", actix_web::web::get().to(git::git))
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
