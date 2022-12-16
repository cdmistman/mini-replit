#[macro_use]
extern crate actix_web;

mod request;

use actix_web::web;
use actix_web::App;
use actix_web::HttpResponse;
use actix_web::HttpServer;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	HttpServer::new(|| App::new().service(eval))
		.bind(("127.0.0.1", 80))?
		.run()
		.await
}

#[post("/eval/{session}/{lang}")]
async fn eval(path: web::Path<(u64, String)>) -> std::io::Result<HttpResponse> {
	todo!()
}
