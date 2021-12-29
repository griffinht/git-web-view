use std::io::BufRead;
use std::io::Read;

struct Parsed { 
    buf: Vec<u8>, 
    tag: Option<String>,
}

pub fn parse(path: &str) -> std::io::Result<&str> {
    let file = std::fs::File::open(format!("{}/directory", path))?;
    
    let mut parse: Vec<Parsed> = Vec::new();
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
        parse.push(Parsed {
            buf: buf,
            tag: Some(String::from_utf8_lossy(tag).to_string())
        });
        buf = Vec::new();
    }
    parse.push(Parsed {
        buf: buf,
        tag: None
    });
    for parse in parse {
        eprint!("{}{}", String::from_utf8_lossy(&parse.buf), match parse.tag {
            Some(tag) => tag,
            None => "blank".to_string()
        });
    }
    eprintln!();
    return Ok("done");
}
