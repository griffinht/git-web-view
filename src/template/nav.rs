pub fn get_nav(request_path: &str) -> Vec<u8> {
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
