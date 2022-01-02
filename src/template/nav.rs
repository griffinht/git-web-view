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
    fn get_directory_link(directory: &str, is_file: bool, i: i32, trailing: &str) -> String {
        if i == 0 { // trailing (current) file/directory should not be a link
            format!("{}{}", directory, trailing)
        } else {
            let string = if i == 1 && is_file {
                "./"
            } else {
                "../"
            };
            format!("<a href=\"{}\">{}{}</a>", get_repeated_string(string, i), directory, trailing)
        }
    }
    let is_bool = !request_path.ends_with("/");
    for directory in request_path.rsplit("/") {
        if directory.is_empty() { continue; }
        // /example/dir/ <- needs to be added back for directories
        nav.push(get_directory_link(directory, is_bool, i, "/"));
        i = i + 1;
    }
    // add leading slash -> /example/dir/
    nav.push(get_directory_link("/", is_bool, i, ""));
    nav.reverse();
    let nav = nav;
    let mut real_nav: Vec<u8> = Vec::new();
    for n in nav {
        real_nav.extend_from_slice(n.as_bytes());
    }
    return real_nav;
}
