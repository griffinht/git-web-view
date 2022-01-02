pub struct File {
    pub path: &'static str,
    pub contents: &'static [u8]
}

pub const STATICS: [File; 2] = [
    File {
        path: "favicon.ico",
        contents: include_bytes!("../default-template/static/favicon.ico")
    }, File {
        path: "style.css",
        contents: include_bytes!("../default-template/static/style.css")
    }
];

pub const TEMPLATES: [File; 2] = [
    File {
        path: "directory.html",
        contents: include_bytes!("../default-template/template/directory.html")
    }, File {
        path:"file.html",
        contents: include_bytes!("../default-template/template/file.html")
    }
];