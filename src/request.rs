#[derive(serde::Deserialize, Debug)]
pub struct EvalRequest {
	pub code: String,
}
