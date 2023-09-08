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
        smart: false,
        default_info_string: None,
        relaxed_tasklist_matching: false,
    }, 
    render: ComrakRenderOptions {
        hardbreaks: true,
        github_pre_lang: false,
        full_info_string: false,
        width: 0,
        unsafe_: false,
        escape: true,
        list_style: ListStyleType::Star,
        sourcepos: false,
    }
};

/// Read a note and return it as a HTTP response.
fn read_note(path: &str) -> HttpResponse {
    let file = fs::read_to_string(path).unwrap_or(String::new());
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
        // '/notes' path
        s if s.starts_with("/notes") => {
            match request.req_type {
                RequestType::GET => read_note(&s[1..]),
                RequestType::POST => write_note(&s[1..], &request.body),

                _ => HttpResponse::not_found()
            }
        }

        _ => HttpResponse::not_found()
    }
}

/// Handle an incoming connection.
fn handle_conn(mut stream: TcpStream) -> io::Result<()> {
    let request = HttpRequest::parse(&mut stream)?;

    eval_request(&request).send(&mut stream)
}
