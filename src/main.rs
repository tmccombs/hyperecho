extern crate hyper;

use std::convert::Infallible;
use std::io::{stdout, Write};
use std::net::SocketAddr;

use http::header::HeaderMap;
use hyper::body::{HttpBody, Sender};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let make_service = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(echo)) });
    let addr: SocketAddr = ([0, 0, 0, 0], 3000).into();
    let server = Server::bind(&addr).serve(make_service);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    } else {
        println!("Listening on port 3000");
    }
}

async fn echo(mut req: Request<Body>) -> Result<Response<Body>, Infallible> {
    // print the request line
    println!("{} {} {:?}\n", req.method(), req.uri(), req.version());
    print_headers(&req.headers());
    print!("\n");

    let (sender, body) = Body::channel();

    tokio::spawn(async move {
        if let Err(err) = do_echo(req.body_mut(), sender).await {
            eprintln!("Error echoing content: {}", err);
        }
    });

    Ok(Response::new(body))
}

async fn do_echo(req: &mut Body, mut resp: Sender) -> Result<(), hyper::Error> {
    let mut out = stdout();
    while let Some(data) = req.data().await {
        let data = data?;
        let _ = out.write_all(&data);
        resp.send_data(data.clone()).await?;
    }
    write!(out, "\n\n").unwrap();
    out.flush().unwrap();
    if let Ok(Some(ref trailers)) = req.trailers().await {
        println!("Trailers:");
        print_headers(trailers);
    }
    Ok(())
}

fn print_headers(headers: &HeaderMap) {
    for (name, value) in headers.iter() {
        println!("{}: {}", name, String::from_utf8_lossy(value.as_bytes()));
    }
}
