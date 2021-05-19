use crate::commands::*;

pub struct SocketResponse {
	pub id: String,
	pub command: APICommand,
	pub data: serde_json::Value
}
