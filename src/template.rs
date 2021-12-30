use std::io::BufRead;
use std::io::Read;

pub struct Parsed { 
    buf: Vec<u8>, 
    tag: Option<String>,
}

pub fn parse_directory(path: &str) -> std::io::Result<std::collections::HashMap<String, Vec<Parsed>>> {
    let mut map: std::collections::HashMap<String, Vec<Parsed>> = std::collections::HashMap::new();
    for file in std::fs::read_dir(path)? {
        let file = file?;
        //todo symlinks and dirs
        map.insert(file.file_name().into_string().unwrap(), match parse(file.path()) {
            Ok(parsed) => { parsed }
            Err(err) => { eprintln!("error parsing {}: {}", file.path().as_os_str().to_str().unwrap(), err); continue; }
        });
    }
    return Ok(map);
}
pub fn parse(path: std::path::PathBuf) -> std::io::Result<Vec<Parsed>> {
    let file = std::fs::File::open(path)?;
    
    let mut parsed: Vec<Parsed> = Vec::new();
    let mut reader = std::io::BufReader::new(file);
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
