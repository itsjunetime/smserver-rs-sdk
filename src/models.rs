use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Conversation {
	pub display_name: String,
	pub chat_identifier: String,
	pub latest_text: String,
	pub has_unread: bool,
	pub addresses: String, // Or maybe vec?
	#[serde(default)]
	pub is_selected: bool
}

impl From<&serde_json::Map<String, serde_json::Value>> for Conversation {
	fn from(val: &serde_json::Map<String, serde_json::Value>) -> Conversation {
		Conversation {
			display_name: val["display_name"].as_str().unwrap_or("Name").to_owned(),
			chat_identifier: val["chat_identifier"].as_str().unwrap_or("").to_owned(),
			latest_text: val["latest_text"].as_str().unwrap_or("").to_owned(),
			has_unread: val["has_unread"].as_bool().unwrap_or(false),
			addresses: val["addresses"].as_str().unwrap_or("").to_owned(),
			is_selected: false,
		}
	}
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

impl From<&serde_json::Map<String, serde_json::Value>> for Message {
	fn from(val: &serde_json::Map<String, serde_json::Value>) -> Message {
		Message {
			guid: val["guid"].as_str().unwrap_or("").to_owned(),
			date: val["date"].as_i64().unwrap_or(0),
			balloon_bundle_id: val["balloon_bundle_id"].as_str().unwrap_or("").to_owned(),
			cache_has_attachments: val["cache_has_attachments"].as_bool().unwrap_or(false),
			imessage: val["service"].as_str().unwrap_or("") == "iMessage",
			is_from_me: val["is_from_me"].as_bool().unwrap_or(false),
			subject: val["subject"].as_str().unwrap_or("").to_owned(),
			text: val["text"].as_str().unwrap_or("").to_owned(),
			associated_message_guid: val["associated_message_guid"].as_str().unwrap_or("").to_owned(),
			associated_message_type: val["associated_message_type"].as_i64().unwrap_or(0) as i16,
			message_type: MessageType::Normal,
			attachments: if val.contains_key("attachments") {
				val["attachments"].as_array()
					.unwrap()
					.iter()
					.map(|a| Attachment::from(a.as_object().unwrap()))
					.collect()
			} else {
				Vec::new()
			},
			date_read: val.get("date_read").map(|d| d.as_i64().unwrap_or(0)),
			sender: val.get("sender").map(|s| s.as_str().unwrap_or("").to_owned()),
			chat_identifier:
				val.get("chat_identifier").map(|c| c.as_str().unwrap_or("").to_owned())
		}
	}
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

impl From<&serde_json::Map<String, serde_json::Value>> for Attachment {
	fn from(val: &serde_json::Map<String, serde_json::Value>) -> Attachment {
		Attachment {
			mime_type: val["mime_type"].as_str().unwrap_or("image/jpeg").to_owned(),
			path: val["filename"].as_str().unwrap_or("file.jpg").to_owned(),
		}
	}
}
