pub fn get(directory: Option<&str>, path: &str) -> Option<Vec<u8>> {
    match directory {
        None => { None }
        Some(directory) => {
            match std::fs::read(format!("{}{}", directory, path)) {
                Ok(template) => { Some(template) }
                Err(_) => { None }
            }
        }
    }
}

pub fn get_static(directory: Option<&str>, path: &str) -> Option<Vec<u8>> {
    match get(directory, path) {
        None => {}
        Some(file) => { return Some(file); }
    }

    Some(match path {
        "favicon.ico" => Vec::from(*include_bytes!("../default-template/static/favicon.ico")),
        "style.css" => Vec::from(*include_bytes!("../default-template/static/style.css")),
        _ => return None
    })
}

pub fn get_template(directory: Option<&str>, path: &str) -> Option<Vec<u8>> {
    match get(directory, path) {
        None => {}
        Some(file) => { return Some(file); }
    }

    Some(match path {
        "directory.html" => Vec::from(*include_bytes!("../default-template/template/directory.html")),
        "file.html" => Vec::from(*include_bytes!("../default-template/template/file.html")),
        _ => return None
    })
}