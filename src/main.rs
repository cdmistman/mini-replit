mod request;
mod response;

use std::collections::HashMap;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Result;

use actix_web::post;
use actix_web::web;
use actix_web::App;
use actix_web::HttpServer;
use rlua::Lua;
use serde_json::to_string;

use self::request::EvalRequest;
use self::response::EvalResponse;

#[actix_web::main]
async fn main() -> Result<()> {
	HttpServer::new(|| {
		App::new()
			.app_data(web::Data::new(Lua::new()))
			.service(eval)
	})
	.bind(("127.0.0.1", 8080))
	.unwrap()
	.run()
	.await
}

#[post("/eval/{session}/{lang}")]
async fn eval(
	path: web::Path<(u64, String)>,
	req: web::Json<EvalRequest>,
	lua: web::Data<Lua>,
) -> web::Json<EvalResponse> {
	let session_id = path.0;
	let lang = path.1.as_str();
	let response = match lang {
		"lua" => lua.context(
			|ctx| match ctx.load(req.code.as_str()).eval::<rlua::Value>() {
				Ok(result) => EvalResponse::from_lua(ctx, result),
				Err(error) => {
					return EvalResponse::Failure {
						error: format!("failed to evaluate code: {error}"),
					}
				},
			},
		),
		_ => EvalResponse::Failure {
			error: format!("language `{lang}` not supported"),
		},
	};
	web::Json(response)
}
