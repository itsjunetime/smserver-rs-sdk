use derive_commands::Commands;
use std::str::FromStr;
use serde::Deserialize;

#[derive(Commands, Deserialize)]
pub enum APICommand {
	#[command(
		subdir = "requests",
		return_type = "Vec<crate::models::Conversation>"
	)]
	#[parameters(chats = "Option<u32>", chats_offset = "Option<u32>")]
	GetChats,

	#[command(return_type = "Vec<crate::models::Message>")]
	#[parameters(
		messages = "&str",
		num_messages = "Option<u32>",
		messages_offset = "Option<u32>",
		read_messages = "Option<bool>",
	)]
	GetMessages,

	#[command(return_type = "String")]
	#[parameters(name = "&str")]
	GetName,

	#[command(subdir = "data", data_return = true)]
	#[parameters(path = "&str")]
	GetAttachment,

	#[command(data_return = true)]
	#[parameters(chat = "&str")]
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
	SendMessage,

	#[parameters(tap_guid = "&str", tapback = "u16", remove_tap = "Option<bool>")]
	SendTapback,

	#[parameters(delete_chat = "&str")]
	DeleteChat,

	#[parameters(delete_text = "&str")]
	DeleteText,

	#[command(rest = false)]
	#[parameters(
		attachment_id = "&str",
		message_id = "&str",
		index = "u32",
		data = "&str",
	)]
	AttachmentData,

	#[parameters(chat = "&str", active = "bool")]
	SendTyping,

	#[data(charging = "bool", percentage = "f64")]
	BatteryStatus,

	#[data(chat = "String", active = "bool")]
	Typing,

	#[data(message = "crate::models::Message")]
	NewMessage,
}
