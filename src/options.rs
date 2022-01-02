pub fn matches(arguments: Vec<String>) -> std::result::Result<std::option::Option<getopts::Matches>, std::io::Error> {

    let mut options = getopts::Options::new();

    options.optflag("h", "help", "print help");
    options.optflag("v", "version", "print version");
    options.optflag("q", "quiet", "don't print verbose debug information like startup and shutdown messages");
    options.optopt("b", "bind", format!("bind to address (default {})", crate::default_bind_address!()).as_str(), "<address>");
    options.optflag("c", "enable-compression", "enable server to compress with deflate, gzip, or brotli when available (takes more cpu)");
    options.optopt("s", "static-directory", "directory containing static files", "<dir>");
    options.optopt("t", "template-directory", "directory containing templates to override default template", "<dir>");
    options.optopt("d", "directory", "directory containing git repository(s)", "<dir>");
    //todo options.optopt("", "base-directory", "base directory for requests, useful for proxied requests", "</location>");

    if arguments.len() == 0 { return Err(std::io::Error::new(std::io::ErrorKind::Other, "missing program name from argv")); }
    let matches = match options.parse(&arguments[1..]) {
        Ok(matches) => matches,
        Err(fail) => { return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("error parsing {}", fail))); }
    };
    if matches.opt_present("h") {
        eprint!("{}", options.usage_with_format(|opts| {
            format!(
                concat!("Usage: ", env!("CARGO_PKG_NAME"), " [options...]\n{}\n"),
                opts.collect::<Vec<String>>().join("\n")
            )
        }));
        return Ok(None)
    }
    if matches.opt_present("v") {
        eprintln!(concat!(env!("CARGO_PKG_NAME"), " version ", env!("CARGO_PKG_VERSION")));
        return Ok(None)
    }

    Ok(Some(matches))
}