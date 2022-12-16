mod request;
mod response;

use std::io::Result;

use actix_web::post;
use actix_web::web;
use actix_web::App;
use actix_web::HttpServer;

use self::request::EvalRequest;
use self::response::EvalResponse;

#[actix_web::main]
async fn main() -> Result<()> {
	HttpServer::new(|| App::new().service(eval))
		.bind(("127.0.0.1", 80))?
		.run()
		.await
}

#[post("/eval/{session}/{lang}")]
async fn eval(
	path: web::Path<(u64, String)>,
	req: web::Json<EvalRequest>,
) -> Result<web::Json<EvalResponse>> {
	todo!()
}
