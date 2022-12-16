use std::collections::HashMap;

use serde::ser::SerializeMap;
use serde::Serializer;

pub enum EvalResponse {
	Success {
		objects: HashMap<Reference, Object>,
		value:   Value,
	},
	Failure {
		error: String,
	},
}

impl serde::Serialize for EvalResponse {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let mut map = serializer.serialize_map(None)?;
		match self {
			EvalResponse::Success { objects, value } => {
				map.serialize_entry("success", &true)?;
				map.serialize_entry("objects", objects)?;
				map.serialize_entry("value", value)?;
			},
			EvalResponse::Failure { error } => {
				map.serialize_entry("success", &false)?;
				map.serialize_entry("error", error)?;
			},
		}
		map.end()
	}
}

pub enum Value {
	Null,
	Number(f64),
	String(String),
	ObjectRef(Reference),
}

impl serde::Serialize for Value {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		match self {
			Value::Null => serializer.serialize_none(),
			Value::Number(n) => serializer.serialize_f64(*n),
			Value::String(s) => serializer.serialize_str(s.as_str()),
			Value::ObjectRef(r) => r.serialize(serializer),
		}
	}
}

#[derive(PartialEq, Eq, Hash)]
pub struct Reference(pub String);

impl serde::Serialize for Reference {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_str(self.0.as_str())
	}
}

#[derive(serde::Serialize)]
pub struct Object {
	pub members: Vec<ObjectMember>,
}

#[derive(serde::Serialize)]
pub struct ObjectMember {
	key:   Value,
	value: Value,
}
