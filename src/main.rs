use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use rust_multithreaded_webserver::ThreadPool;

const MAX_THREADS: usize = 4;

// HTTP Request Format:
// Method Request-URI HTTP-Version CRLF
// headers CRLF
// message-body
//
//
// HTTP Response Format:
// HTTP-Version Status-Code Reason-Phrase CRLF
// headers CRLF
// message-body
//
//

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    // just read the first line
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    // need to match on the full slice of the request line as the
    // match will not pattern match against string literal values.
    // match does not do automatic referencing and dereferencing.
    let (status_line, filename) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "index.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "index.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
    };

    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();

    // println!("Request: {:#?}", http_request);
}
fn main() {
    let address_and_port = String::from("127.0.0.1:7878");
    println!("Listening on {}", address_and_port);
    let listener = TcpListener::bind(address_and_port).unwrap();

    println!("Using {MAX_THREADS} threads.");
    let thread_pool = ThreadPool::new(MAX_THREADS);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        println!("Connection established.");

        thread_pool.execute(|| {
            handle_connection(stream);
        });
        // stream will close the connection when it goes out of scope
    }
}
