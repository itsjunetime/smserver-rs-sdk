use crate::commands::*;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct SocketResponse {
	pub id: String,
	pub command: APICommand,
	pub data: serde_json::Value
}
