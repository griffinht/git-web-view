pub fn get_links(path: &String) -> std::io::Result<Vec<u8>> {
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
