use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Conversation {
	pub display_name: String,
	pub chat_identifier: String,
	pub latest_text: String,
	pub has_unread: bool,
	pub addresses: String, // Or maybe vec?
	#[serde(default)]
	pub is_selected: bool,
	#[serde(default)]
	pub pinned: bool,
}

#[derive(Debug, Deserialize)]
pub struct Message {
	pub guid: String,
	pub date_read: Option<i64>,
	pub date: i64,
	pub balloon_bundle_id: String,
	pub cache_has_attachments: bool,
	#[serde(default)]
	pub attachments: Vec<Attachment>,
	pub imessage: bool,
	pub is_from_me: bool,
	pub subject: String,
	pub text: String,
	pub associated_message_guid: String,
	pub associated_message_type: i16,
	pub sender: Option<String>,
	pub chat_identifier: Option<String>,
	#[serde(default)]
	pub message_type: MessageType,
}

impl Message {
	pub fn typing(chat: &str) -> Message {
		Message {
			guid: "".to_owned(),
			date: 0,
			balloon_bundle_id: "".to_owned(),
			cache_has_attachments: false,
			imessage: true,
			is_from_me: false,
			subject: "".to_owned(),
			text: "".to_owned(),
			associated_message_guid: "".to_owned(),
			associated_message_type: 0,
			message_type: MessageType::Typing,
			chat_identifier: Some(chat.to_owned()),
			attachments: Vec::new(),
			date_read: None,
			sender: None,
		}
	}

	pub fn idle(chat: &str) -> Message {
		Message {
			guid: "".to_owned(),
			date: 0,
			balloon_bundle_id: "".to_owned(),
			cache_has_attachments: false,
			imessage: true,
			is_from_me: false,
			subject: "".to_owned(),
			text: "".to_owned(),
			associated_message_guid: "".to_owned(),
			associated_message_type: 0,
			message_type: MessageType::Idle,
			chat_identifier: Some(chat.to_owned()),
			attachments: Vec::new(),
			date_read: None,
			sender: None,
		}
	}
}

#[derive(PartialEq, Debug, Deserialize)]
pub enum MessageType {
	Normal,
	Typing,
	Idle,
}

impl Default for MessageType {
	fn default() -> Self {
		MessageType::Normal
	}
}

#[derive(Debug, Deserialize)]
pub struct Attachment {
	pub mime_type: String,
	#[serde(rename = "filename")]
	pub path: String,
}
