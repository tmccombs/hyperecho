extern crate hyper;

use std::io::prelude::*;
use std::io::{stdout, Result};

use hyper::header::Headers;
use hyper::server::{Server, Request, Response};



fn main() {
  Server::http("0.0.0.0:3000").and_then(|server| {
    server.handle(echo_handler)
  }).expect("Unable to listen on port 3000");
}

fn echo_handler(req: Request, resp: Response) {
  match do_echo(req, resp) {
    Ok(_) => {},
    Err(e) => {
      println!("Error: {}\n", e);
    }
  }
}

fn do_echo(mut req: Request, resp: Response) -> Result<()> {
  // print the request line
  println!("{} {} {}\n", req.method, req.uri, req.version);
  print_headers(&req.headers);
  print!("\n");

  let mut buffer: [u8; 1024] = [0; 1024];
  let mut response = try!(resp.start());
  let mut out = stdout();
  while let Ok(count) = req.read(&mut buffer) {
    if count == 0 {
      break;
    }
    let chunk = &buffer[0..count];
    try!(response.write_all(chunk));
    try!(out.write_all(chunk));
  }
  print!("\n\n");
  try!(out.flush());
  response.end()
}

fn print_headers(headers: &Headers) {
  for header in headers.iter() {
    println!("{}: {}", header.name(), header.value_string());
  }
}
