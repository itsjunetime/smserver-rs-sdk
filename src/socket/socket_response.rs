use crate::commands::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct SocketResponse {
	#[serde(default)]
	pub id: String,
	pub last: bool,
	pub command: APICommand,
	pub data: serde_json::Value
}
