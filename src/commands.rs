use derive_commands::Commands;
use std::str::FromStr;
use serde::Deserialize;

#[derive(Commands, Deserialize, Debug)]
pub enum APICommand {
	#[command(
		subdir = "requests",
		return_type = "Vec<crate::models::Conversation>"
	)]
	#[parameters(chats = "Option<u32>", chats_offset = "Option<u32>")]
	#[serde(rename = "get-chats")]
	GetChats,

	#[command(return_type = "Vec<crate::models::Message>")]
	#[parameters(
		messages = "&str",
		num_messages = "Option<u32>",
		messages_offset = "Option<u32>",
		read_messages = "Option<bool>",
	)]
	#[serde(rename = "get-messages")]
	GetMessages,

	#[command(return_type = "String")]
	#[parameters(name = "&str")]
	#[serde(rename = "get-name")]
	GetName,

	#[command(subdir = "data", data_return = true)]
	#[parameters(path = "&str")]
	#[serde(rename = "get-attachment")]
	GetAttachment,

	#[command(data_return = true)]
	#[parameters(chat = "&str")]
	#[serde(rename = "get-icon")]
	GetIcon,

	#[command(
		subdir = "send",
		multipart = true,
		files = "attachments",
		no_main = true
	)]
	#[parameters(
		chat = "String",
		text = "Option<String>",
		subject = "Option<String>",
		attachments = "Option<serde_json::Value>",
		photos = "Option<String>"
	)]
	#[serde(rename = "send-message")]
	SendMessage,

	#[parameters(tap_guid = "&str", tapback = "u16", remove_tap = "Option<bool>")]
	#[serde(rename = "send-tapback")]
	SendTapback,

	#[parameters(delete_chat = "&str")]
	#[serde(rename = "delete-chat")]
	DeleteChat,

	#[parameters(delete_text = "&str")]
	#[serde(rename = "delete-text")]
	DeleteText,

	#[command(rest = false)]
	#[parameters(
		attachment_id = "&str",
		message_id = "&str",
		index = "u32",
		data = "&str",
	)]
	#[serde(rename = "attachment-data")]
	AttachmentData,

	#[parameters(chat = "&str", active = "bool")]
	#[serde(rename = "send-typing")]
	SendTyping,

	#[data(charging = "bool", percentage = "f64")]
	#[serde(rename = "battery-status")]
	BatteryStatus,

	#[data(chat = "String", active = "bool")]
	#[serde(rename = "typing")]
	Typing,

	#[data(message = "crate::models::Message")]
	#[serde(rename = "new-message")]
	NewMessage,
}
