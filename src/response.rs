mod lua;

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

#[derive(PartialEq, Debug)]
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
			Value::Number(n) => {
				let mut map = serializer.serialize_map(Some(2))?;
				map.serialize_entry("kind", "number")?;
				map.serialize_entry("value", n)?;
				map.end()
			},
			Value::String(s) => {
				let mut map = serializer.serialize_map(Some(2))?;
				map.serialize_entry("kind", "string")?;
				map.serialize_entry("value", s.as_str())?;
				map.end()
			},
			Value::ObjectRef(r) => {
				let mut map = serializer.serialize_map(Some(2))?;
				map.serialize_entry("kind", "ref")?;
				map.serialize_entry("value", r)?;
				map.end()
			},
		}
	}
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Reference(pub String);

impl serde::Serialize for Reference {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_str(self.0.as_str())
	}
}

#[derive(serde::Serialize, PartialEq, Debug)]
pub struct Object {
	pub members: Vec<ObjectMember>,
}

#[derive(serde::Serialize, PartialEq, Debug)]
pub struct ObjectMember {
	pub key:   Value,
	pub value: Value,
}

#[cfg(test)]
mod tests {
	use std::collections::HashMap;

	use super::*;

	#[rstest::rstest]
	#[case(
		EvalResponse::Failure {
			error: "uh oh".to_string(),
		},
		r#"{"success":false,"error":"uh oh"}"#,
	)]
	#[case(
		EvalResponse::Success {
			objects: HashMap::new(),
			value: Value::Null,
		},
		r#"{"success":true,"objects":{},"value":null}"#,
	)]
	#[case(
		EvalResponse::Success {
			objects: HashMap::from([
				(Reference("01".to_string()), Object {
					members: vec![]
				})
			]),
			value: Value::ObjectRef(Reference("01".to_string()))
		},
		r#"{"success":true,"objects":{"01":{"members":[]}},"value":{"kind":"ref","value":"01"}}"#,
	)]
	// more complex test: mutually-recursive objects
	// NOTE: serialization of the `objects` hashmap is inconsistent, so sometimes this test fails :/
	#[case(
		EvalResponse::Success {
			objects: HashMap::from([
				(Reference("x".to_string()), Object {
					members: vec![
						ObjectMember {
							key: Value::ObjectRef(Reference("y".to_string())),
							value: Value::ObjectRef(Reference("y".to_string())),
						},
					]
				}),
				(Reference("y".to_string()), Object {
					members: vec![
						ObjectMember {
							key: Value::ObjectRef(Reference("x".to_string())),
							value: Value::ObjectRef(Reference("x".to_string())),
						}
					]
				})
			]),
			value: Value::ObjectRef(Reference("x".to_string())),
		},
		r#"{"success":true,"objects":{"y":{"members":[{"key":{"kind":"ref","value":"x"},"value":{"kind":"ref","value":"x"}}]},"x":{"members":[{"key":{"kind":"ref","value":"y"},"value":{"kind":"ref","value":"y"}}]}},"value":{"kind":"ref","value":"x"}}"#,
	)]
	fn test(#[case] input: EvalResponse, #[case] expect: &str) {
		assert_eq!(serde_json::to_string(&input).unwrap().as_str(), expect)
	}
}
