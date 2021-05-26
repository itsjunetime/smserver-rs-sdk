use crate::commands::*;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct SocketResponse {
	#[serde(default)]
	pub id: String,
	pub command: APICommand,
	pub data: serde_json::Value
}
