use std::collections::BTreeMap;

use tokio::sync::Mutex;
use tokio::sync::RwLock;

pub type Sessions = RwLock<BTreeMap<String, Mutex<BTreeMap<String, SessionLang>>>>;

#[derive(Debug)]
pub enum SessionLang {
	Lua(rlua::Lua),
}
