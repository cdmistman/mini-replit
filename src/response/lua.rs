use super::*;

impl EvalResponse {
	pub fn from_lua<'lua>(lua_ctx: rlua::Context<'lua>, lua_value: rlua::Value<'lua>) -> Self {
		let tostring: rlua::Function = lua_ctx.globals().get("tostring").unwrap();
		let mut objects = HashMap::new();
		let mut stack = Vec::new();

		let result = match conv_value(tostring.clone(), &mut stack, lua_value) {
			Ok(r) => r,
			Err(error) => return EvalResponse::Failure { error },
		};
		while let Some((reference, table)) = stack.pop() {
			let mut members = Vec::new();
			for table_entry in table.pairs::<rlua::Value, rlua::Value>() {
				let (lua_key, lua_value) = match table_entry {
					Ok(p) => p,
					Err(error) => {
						return EvalResponse::Failure {
							error: format!("unable to serialize object: {error}"),
						}
					},
				};
				let key = match conv_value(tostring.clone(), &mut stack, lua_key) {
					Ok(r) => r,
					Err(error) => return EvalResponse::Failure { error },
				};
				let value = match conv_value(tostring.clone(), &mut stack, lua_value) {
					Ok(r) => r,
					Err(error) => return EvalResponse::Failure { error },
				};
				members.push(ObjectMember { key, value })
			}
			objects.insert(reference, Object { members });
		}

		return EvalResponse::Success {
			objects,
			value: result,
		};
	}
}

fn conv_value<'lua>(
	tostring: rlua::Function<'lua>,
	stack: &mut Vec<(Reference, rlua::Table<'lua>)>,
	value: rlua::Value<'lua>,
) -> Result<Value, String> {
	Ok(match value {
		rlua::Value::Nil => Value::Null,
		rlua::Value::Integer(i) => Value::Number(i as f64),
		rlua::Value::Number(n) => Value::Number(n),
		rlua::Value::String(s) => Value::String(s.to_str().unwrap().to_string()),
		rlua::Value::Table(new_t) => {
			let new_t_name = Reference(tostring.call(new_t.clone()).unwrap());
			stack.push((new_t_name.clone(), new_t));
			Value::ObjectRef(new_t_name)
		},
		_ => todo!(),
	})
}

#[cfg(test)]
mod tests {
	use super::*;

	#[rstest::fixture]
	fn lua() -> rlua::Lua {
		rlua::Lua::new()
	}

	fn no_objects() -> HashMap<Reference, Object> {
		HashMap::new()
	}

	fn ref_x() -> Reference {
		Reference("x".to_string())
	}

	#[rstest::rstest]
	#[case("nil", no_objects(), Value::Null)]
	#[case("1", no_objects(), Value::Number(1.0))]
	#[case("1.1", no_objects(), Value::Number(1.1))]
	#[case(r#""hello, world!""#, no_objects(), Value::String("hello, world!".to_string()))]
	#[case(r#"x = {}; return x"#, HashMap::from([(ref_x(), Object { members: vec![] })]), Value::ObjectRef(ref_x()))]
	#[case(r#"x = {'foo': 2, 'bar': nil}; return x"#, HashMap::from([(ref_x(), Object { members: vec![
		ObjectMember {
			key: Value::String("foo".to_string()),
			value: Value::Number(2.0),
		},
		ObjectMember {
			key: Value::String("bar".to_string()),
			value: Value::Null,
		}
	] })]), Value::ObjectRef(ref_x()))]
	fn eval(
		lua: rlua::Lua,
		#[case] input: &str,
		#[case] expect_objects: HashMap<Reference, Object>,
		#[case] expect_value: Value,
	) {
		let response = lua.context(|ctx| match ctx.load(input).eval::<rlua::Value>() {
			Ok(result) => EvalResponse::from_lua(ctx, result),
			Err(error) => panic!("can't execute: `{error}`"),
		});

		let (actual_objects, actual_value) = match response {
			EvalResponse::Success { objects, value } => (objects, value),
			EvalResponse::Failure { error } => panic!("can't convert: `{error}`"),
		};
		assert_eq!(actual_objects.len(), expect_objects.len());

		if let Value::ObjectRef(original_name) = expect_value {
			todo!()
		} else {
			assert_eq!(actual_value, expect_value);
		}
	}
}
