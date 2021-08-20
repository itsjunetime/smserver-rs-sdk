use derive_commands::Commands;
use std::str::FromStr;
use serde::Deserialize;

#[derive(Commands, Deserialize, Debug)]
pub enum APICommand {
	// so basically the entire API is created through this enum and the `Commands`
	// macro that I wrote. For each variant, it does three main things:
	// 1. Unless, within the commands attribute, `rest` is set to false or the
	//       variant has no `parameters` attribute, it creates a function to send
	//       this data through the RestAPIClient, using the parameters defined in
	//       the `parameters` attribute.
	//
	//       If `multipart` is set to true, it sends it as a multipart form, with
	//       the key defined by `files` taking a Vec<String>, and being the files
	//       that are sent through the form. If `multipart` is not set or set to
	//       false, it sends it as a GET URL Query.
	//
	//       Unless `data_return` is true, the RestAPIClient function returns a
	//       value of the type defined by `return_type`. If `data_return` is true,
	//       it returns a Vec<u8> -- the data.
	//
	//       You can also change which subdirectory of the rest_base_url the
	//       command goes to with the `subdir` key in the `command` attribute.
	//
	// 2. Unless, within the commands attribute, `socket` is set to false or it has
	//       no `parameters` attribute, it creates a function to send the data
	//       defined in `parameters` through the socket, for the purpose described
	//       by the variant name.
	//
	//       Unless `data_return` is true, the RestAPIClient function returns a
	//       value of the type defined by `return_type`. If `data_return` is true,
	//       it returns a Vec<u8> (the data).
	//
	// 3. If 1 and 2 don't happen & the `no_main` attribute is not set, this macro
	//       creates a function to perform this function from the `APIClient`
	//       struct. The function automatically checks whether the SDK is
	//       connecting via REST or remote websocket, and then calls the
	//       appropriate function.
	//
	// If a variant has a `data` attribute, that means that this data cannot be
	// sent to the host and the client can only receive this information from the
	// host, through the socket. If this is the case:
	// - The macro will create a struct which has the fields defined in `data`,
	//   plus `id: String` and `command: APICommand`.
	//
	// - The struct's name will be `${Variant}Notification`, e.g. BatteryStatus =>
	//   `BatteryStatusNotification`.
	//
	// - A function will also be created which automatically converts a
	//   `SocketResponse` into the generated struct, consuming the SocketResponse
	//   in the process.
	//
	// This macro also creates an `impl` of APICommand that allows you to get the
	// command string for each variant (e.g. GetChats => "get-chats")

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

	#[command(return_type = "crate::models::Conversation")]
	#[parameters(chat_id = "&str")]
	#[serde(rename = "get-conversation")]
	GetConversation,

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

	#[command(return_type = "Vec<crate::models::Photo>")]
	#[parameters(photos = "Option<u32>", photos_offset = "Option<u32>", photos_recent = "Option<bool>")]
	#[serde(rename = "get-photos")]
	GetPhotos,

	#[command(data_return = true)]
	#[parameters(photo = "&str")]
	#[serde(rename = "get-photo")]
	GetPhoto,

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
