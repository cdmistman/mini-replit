mod request;
mod response;
mod session;

use std::io::Result;

use actix_web::post;
use actix_web::web;
use actix_web::App;
use actix_web::HttpServer;
use rlua::Lua;
use uuid::Uuid;

use self::request::EvalRequest;
use self::response::EvalResponse;
use self::session::SessionLang;
use self::session::Sessions;

#[actix_web::main]
async fn main() -> Result<()> {
	let sessions = web::Data::new(Sessions::default());

	HttpServer::new(move || App::new().app_data(sessions.clone()).service(eval))
		.bind(("127.0.0.1", 8080))
		.unwrap()
		.run()
		.await
}

#[post("/new")]
async fn new_session(sessions: web::Data<Sessions>) -> web::Json<String> {
	let mut sessions = sessions.write().await;
	let mut session_id = Uuid::new_v4().to_string();
	while sessions.contains_key(session_id.as_str()) {
		session_id = Uuid::new_v4().to_string();
	}
	sessions.insert(session_id.clone(), Default::default());
	web::Json(session_id)
}

#[post("/eval/{session}/{lang}")]
async fn eval(
	path: web::Path<(String, String)>,
	req: web::Json<EvalRequest>,
	sessions: web::Data<Sessions>,
) -> web::Json<EvalResponse> {
	let session_id = path.0.as_str();
	let lang = path.1.as_str();
	let response = match lang {
		"lua" => {
			let sessions = sessions.read().await;
			let mut session_langs = sessions[session_id].lock().await;
			if !session_langs.contains_key("lua") {
				session_langs.insert("lua".to_string(), SessionLang::Lua(Lua::new()));
			}

			#[allow(irrefutable_let_patterns)]
			let SessionLang::Lua(lua) = &session_langs["lua"] else {
				unreachable!();
			};

			lua.context(
				|ctx| match ctx.load(req.code.as_str()).eval::<rlua::Value>() {
					Ok(result) => EvalResponse::from_lua(ctx, result),
					Err(error) => {
						return EvalResponse::Failure {
							error: format!("failed to evaluate code: {error}"),
						}
					},
				},
			)
		},
		_ => EvalResponse::Failure {
			error: format!("language `{lang}` not supported"),
		},
	};
	web::Json(response)
}
