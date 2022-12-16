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
