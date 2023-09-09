use std::{net::{TcpListener, TcpStream}, io::{self}, fs};

use comrak::{markdown_to_html, ComrakOptions, ComrakRenderOptions, ComrakParseOptions, ComrakExtensionOptions, ListStyleType};
use http::{HttpResponse, HttpRequest, RequestType};

mod http;

/// HTTP server host address.
const ADDR: &'static str = "127.0.0.1:1440";

fn main() {
    let listener = TcpListener::bind(ADDR).unwrap();

    println!("\r\n~ Note Server");
    println!("Waiting for requests at {}", ADDR);

    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            _ = handle_conn(stream);
        }
    }
}

const COMRAK_OPTIONS: ComrakOptions = ComrakOptions { 
    extension: ComrakExtensionOptions {
        strikethrough: true,
        tagfilter: false,
        table: true,
        autolink: false,
        tasklist: true,
        superscript: true,
        header_ids: Some(String::new()),
        footnotes: true,
        description_lists: false,
        front_matter_delimiter: None,
    }, 
    parse: ComrakParseOptions {
        smart: true,
        default_info_string: None,
        relaxed_tasklist_matching: false,
    }, 
    render: ComrakRenderOptions {
        hardbreaks: true,
        github_pre_lang: false,
        full_info_string: false,
        width: 0,
        unsafe_: true,
        escape: false,
        list_style: ListStyleType::Star,
        sourcepos: false,
    }
};

/// Read a note and return it as a HTTP response.
fn read_note(path: &str) -> HttpResponse {
    let Ok(file) = fs::read_to_string(path) else {
        return HttpResponse::not_found();
    };
    let mut response = HttpResponse::ok();

    response.html(&markdown_to_html(&file, &COMRAK_OPTIONS));
    response
}

/// Write a note and return it as a HTTP response.
fn write_note(path: &str, body: &str) -> HttpResponse {
    if path.contains("..") {
        return HttpResponse::err_with_context("'..' is not allowed in note paths.");
    };

    if let Err(_) = fs::write(path, body) {
        HttpResponse::err_with_context("Failed to save note file.")
    } else {
        read_note(path)
    }
}

/// Evaluate an incoming HTTP request.
fn eval_request(request: &HttpRequest) -> HttpResponse {
    match &request.path {
        /* read or write a note */
        s if s.starts_with("/notes") => {
            match request.req_type {
                RequestType::GET => read_note(&s[1..]),
                RequestType::POST => write_note(&s[1..], &request.body),

                _ => HttpResponse::not_found()
            }
        }
        
        /* list all notes */
        s if s.starts_with("/list") => {
            if s.len() <= 6 || !s.contains('?') {
                return HttpResponse::err_with_context("Missing query string '?<start>:<end>'");
            }

            let mut bounds = s[6..].split(':');
            let Some(start) = bounds.next() else {
                return HttpResponse::err_with_context("Missing start bounds '?<start>:<end>'");
            };
            let Some(end) = bounds.next() else {
                return HttpResponse::err_with_context("Missing end bounds '?<start>:<end>'");
            };

            let Ok(start) = start.parse::<u16>() else {
                return HttpResponse::err_with_context("Start bounds is not a valid number");
            };
            let Ok(end) = end.parse::<u16>() else {
                return HttpResponse::err_with_context("End bounds is not a valid number");
            };

            if start > end {
                return HttpResponse::err_with_context("Start of the bounds is bigger then the end");
            }

            let mut notes: Vec<String> = Vec::new();
            let mut paths = fs::read_dir("notes").unwrap();
            let mut i = 0;

            while let Some(Ok(entry)) = paths.next() {
                if i < start || i >= end {
                    i += 1;
                    continue;
                }

                notes.push(entry.file_name().to_string_lossy().to_string());
                i += 1;
            }

            let mut response = HttpResponse::ok();
            response.json(&format!("[\"{}\"]", notes.join("\", \"")));
            response
        }

        _ => HttpResponse::not_found()
    }
}

/// Handle an incoming connection.
fn handle_conn(mut stream: TcpStream) -> io::Result<()> {
    let request = HttpRequest::parse(&mut stream)?;

    eval_request(&request).send(&mut stream)
}
