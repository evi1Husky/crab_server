use std::{
    ffi::OsStr,
    fs,
    io::{self, BufRead, BufReader, Result, Write},
    net::TcpStream,
    path::Path,
};

use crate::mime_types::mime_types;
use crate::not_found::NOT_FOUND;

fn get_request_line(mut stream: &TcpStream) -> Option<String> {
    BufReader::new(&mut stream)
        .lines()
        .next()
        .and_then(|x| match x {
            Ok(x) => {
                if x.starts_with("GET /") & x.contains(" HTTP/") {
                    Some(x)
                } else {
                    None
                }
            }
            Err(_) => None,
        })
}

fn get_path(req: String) -> Option<String> {
    req.split_whitespace().nth(1).and_then(|x| {
        if x.contains('.') & x.starts_with('/') {
            Some(x.trim_start_matches('/').to_owned())
        } else {
            None
        }
    })
}

fn get_mime(path: String) -> Option<String> {
    Path::new(&path)
        .extension()
        .and_then(OsStr::to_str)
        .and_then(mime_types)
}

fn load_file(path: String) -> Result<String> {
    fs::read_to_string(path)
}

fn add_page_reloader(mut file: String) -> String {
    file.push_str(
        r#"
<script>
  fetch("http://localhost:8087", {
    method: "GET",
    headers: {
      "Content-Type": "text/plain",
      "accept": "text/plain",
    },
      mode: "no-cors",
  })
  .then(x => x.text())
  .then(() => location.reload())
  .catch(x => x);
</script>"#,
    );
    file
}

fn add_favicon(file: String, fav_icon: &str) -> String {
    file.replace(
        "<head>",
        &format!(
            "<head>\n
<link rel='icon' href='data:image/svg+xml,
<svg xmlns=%22http://www.w3.org/2000/svg%22 viewBox=%220 0 100 100%22>
<text y=%22.9em%22 font-size=%2290%22>{fav_icon}</text></svg>' />"
        ),
    )
}

pub fn process(mut stream: TcpStream, fav_icon: &str) -> io::Result<()> {
    let path = get_request_line(&stream).and_then(get_path);

    let mime = match path.clone() {
        Some(x) => get_mime(x),
        None => None,
    };

    let file = match mime {
        Some(_) => match load_file(path.unwrap_or_default()) {
            Ok(x) => Some(x),
            Err(_) => None,
        },
        None => None,
    };

    let (mut file, status_line, mime) = match file {
        Some(x) => (x, "HTTP/1.1 200 OK", mime.unwrap_or_default()),
        None => (
            String::from(NOT_FOUND),
            "HTTP/1.1 404 NOT FOUND",
            String::from("text/html"),
        ),
    };

    if mime.contains("text/html") {
        file = add_page_reloader(file);
        file = add_favicon(file, fav_icon);
    }

    let len = file.len();

    let response = format!(
        "\
        {status_line}\r\n\
        Access-Control-Allow-Origin: *\r\n\
        Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS\r\n\
        Cache-Control: no-cache\r\n\
        Content-Type: {mime}; charset=UTF-8\r\n\
        Content-Length: {len}\r\n\r\n\
        {file}"
    );

    stream.write_all(response.as_bytes())?;

    Ok(())
}
