extern crate hyper;

use std::convert::Infallible;
use std::io::{stdout, Write};
use std::net::SocketAddr;

use http::header::HeaderMap;
use http_body_util::BodyExt;
use hyper::body::{Body, Bytes, Incoming};
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use hyper_util::server::conn::auto;

use tokio::net::TcpListener;

#[cfg(not(any(feature = "http1", feature = "http2")))]
const _: () = assert!(false, "Must enable at least one of http1 and http2");

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<Infallible, std::io::Error> {
    let addr: SocketAddr = ([0, 0, 0, 0], 3000).into();
    let listener = TcpListener::bind(addr).await?;

    let builder = auto::Builder::new(hyper_util::rt::TokioExecutor::new());

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let builder = builder.clone();

        tokio::task::spawn(async move {
            if let Err(err) = builder.serve_connection(io, service_fn(echo)).await {
                eprintln!("server error: {:?}", err);
            }
        });
    }
}

async fn echo(
    req: Request<Incoming>,
) -> Result<Response<impl Body<Data = Bytes, Error = hyper::Error>>, Infallible> {
    // print the request line
    println!("\n{} {} {:?}\n", req.method(), req.uri(), req.version());
    print_headers(req.headers());
    println!();

    let body = req.into_body().map_frame(|frame| {
        let mut out = stdout();
        if let Some(data) = frame.data_ref() {
            out.write_all(data).unwrap();
            out.flush().unwrap();
        }
        if let Some(trailers) = frame.trailers_ref() {
            print_headers(trailers);
        }
        frame
    });

    Ok(Response::new(body))
}

fn print_headers(headers: &HeaderMap) {
    for (name, value) in headers.iter() {
        println!("{}: {}", name, String::from_utf8_lossy(value.as_bytes()));
    }
}
