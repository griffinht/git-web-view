use std::io::BufRead;

pub mod nav;
pub mod links;

#[derive(Clone)]
pub struct Parsed {
    pub(crate) buf: Vec<u8>,
    pub(crate) tag: Option<String>,
}

pub fn parse_directory_default() -> std::collections::HashMap<String, Vec<Parsed>> {
    let mut map: std::collections::HashMap<String, Vec<Parsed>> = std::collections::HashMap::new();
    fn doo(map: &mut std::collections::HashMap<String, Vec<Parsed>>, name: String, bytes: &[u8]) {
        match parse(std::io::BufReader::new(bytes)) {
            Ok(parse) => { map.insert(name, parse); }
            Err(err) => { eprintln!("error parsing: {}", err); }
        };
    }
    doo(&mut map, "directory.html".parse().unwrap(), include_bytes!("../default-template/directory.html"));
    doo(&mut map, "file.html".parse().unwrap(), include_bytes!("../default-template/file.html"));
    return map;
}

pub fn parse_directory(path: &str) -> std::collections::HashMap<String, Vec<Parsed>> {
    let mut map: std::collections::HashMap<String, Vec<Parsed>> = std::collections::HashMap::new();
    for file in match std::fs::read_dir(path) {
        Ok(file) => file,
        Err(err) => { eprintln!("err reading directory {}: {}", path, err); return map; }
    } {
        let file = match file {
            Ok(file) => file,
            Err(err) => { eprintln!("err reading {}: {}", path, err); continue; }
        };
        //todo symlinks and dirs
        map.insert(file.file_name().into_string().unwrap(), match parse_file(file.path()) {
            Ok(parsed) => { parsed }
            Err(err) => { eprintln!("error parsing {}: {}", file.path().as_os_str().to_str().unwrap(), err); continue; }
        });
    }
    map
}
fn parse_file(path: std::path::PathBuf) -> std::io::Result<Vec<Parsed>> {
    let file = std::fs::File::open(path)?;
    parse(std::io::BufReader::new(file))
}
pub fn parse<T: BufRead>(mut reader: T) -> std::io::Result<Vec<Parsed>> {
    let mut parsed: Vec<Parsed> = Vec::new();
    let mut buf: Vec<u8> = Vec::new();
    loop {
        let read = reader.read_until('$' as u8, &mut buf)?;
        
        if read == 0 { break; }

        let open = &mut [0 as u8; 1];
        match reader.read_exact(open) {
            Ok(_) => {},
            Err(err) => {
                if err.kind() == std::io::ErrorKind::UnexpectedEof {
                    break;
                }
                return Err(err);
            }
        };
        if open[0] != '{' as u8 {
            buf.push(open[0]); // not a tag, so add back the old content
            continue;
        }
        // now we know it is a tag
        buf.pop(); // remove $
        let mut tag: Vec<u8> = Vec::new();
        let read = reader.read_until('}' as u8, &mut tag)?;
        let tag = &tag[0..read - 1];
        parsed.push(Parsed {
            buf: buf,
            tag: Some(String::from_utf8_lossy(tag).to_string())
        });
        buf = Vec::new();
    }
    parsed.push(Parsed {
        buf: buf,
        tag: None
    });
    return Ok(parsed);
}
