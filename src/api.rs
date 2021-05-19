use std::{
	collections::HashMap,
	sync::{
		mpsc,
		Arc,
		RwLock
	}
};
use crate::{
	rest_api::RestAPIClient,
	socket::*,
	config::*,
};
use serde_json::json;
use tokio_tungstenite::tungstenite::{
	Error,
	Message
};

pub struct APIClient {
	pub rest_client: RestAPIClient,
	pub socket: SocketHandler,
	pub sock_msgs: Arc<RwLock<HashMap<String, mpsc::Sender<SocketResponse>>>>,
	pub uses_rest: bool,
	pub chunk_size: usize,
}

impl APIClient {
	// so when this struct is initialized, it needs to pass a clone of the
	// sock_msgs hashmap & notif_rec receiver into the spawn_receiver function
	// of the socket handler.
	//
	// Then, every time the socket handler receives a new message,
	// it automatically grabs the mpsc::Sender that relates to the id of the msg.
	// It sends the socket response through the sender, which is received by
	// the receiver who is awaiting a message.
	//
	// If there is no sender, it just sends the data through an mpsc::sender that
	// the user will have passed in when they created this
	//
	// This is the template for how each of the API communication functions
	// in this struct will look.
	/*
	pub async fn do_command(
		&mut self, param: String
	) -> anyhow::Result<DoCommandResponse> {
		if self.uses_rest {
			return self.rest_client.do_command(param);
		}

		let id = self.socket.do_command(param).await?;
		let (sender, receiver) = mpsc::channel();

		self.sock_msgs.insert(id, sender);

		Ok(receiver.recv()?.do_command_data())

		if let Ok(msg) = receiver.recv() {
			let parsed_res: DoCommandResponse = msg.do_command_data();
			Ok(parsed_res)
			// or Ok(msg.do_command_data())
		}

		Err(SDKError::MangledSend.into())
	}
	*/

	pub async fn new(config: SDKConfig, sender: mpsc::Sender<Result<Message, Error>>) -> anyhow::Result<APIClient> {
		let chunk_size = config.chunk_size;
		let uses_rest = config.use_rest;
		let secure = config.secure;
		let base_url = config.rest_base_url.to_owned();

		let rest_client = RestAPIClient::new(config);
		let sock_msgs = Arc::new(RwLock::new(HashMap::new()));

		let url = url::Url::parse(&base_url)?;

		let port = url.port().unwrap_or(8741);
		let host = match url.host() {
			None => "".to_owned(),
			Some(h) => h.to_string()
		};

		let socket = SocketHandler::new(port as u32, &host, secure, Some(url.path()), sender).await?;

		Ok(APIClient{
			rest_client,
			socket,
			sock_msgs,
			uses_rest,
			chunk_size
		})
	}

	pub async fn send_message(
		&mut self,
		chat: String,
		text: Option<String>,
		subject: Option<String>,
		attachments: Option<Vec<String>>,
		photos: Option<Vec<String>>,
	) -> anyhow::Result<()> {
		let photos_str = match photos {
			Some(phs) => Some(phs.join(":")),
			None => None
		};

		if self.uses_rest {
			return self.rest_client
				.send_message(chat, text, subject, photos_str, attachments)
				.await;
		}

		let (datas, mut infos): (Vec<Vec<u8>>, Vec<(u32, String)>) = match attachments {
			None => (vec![Vec::new()], Vec::new()),
			Some(ref files) => files.iter().fold(
				(Vec::new(), Vec::new()), | (mut d, mut i), f | {
					let bin = match std::fs::read(f) {
						Ok(bin) => bin,
						Err(_) => Vec::new()
					};

					d.push(bin);

					let id = uuid::Uuid::new_v4().to_string();
					let size = match std::fs::metadata(f) {
						Ok(meta) => meta.len(),
						Err(_) => 0,
					};

					let len = (size as f64 / self.chunk_size as f64).ceil() as u32;

					i.push((len, id));

					(d, i)
				}),
		};

		let json_info = infos.iter()
			.zip(attachments.unwrap_or(Vec::new()))
			.map(
				| (i, a) | {
					json!({
						"size": i.0,
						"id": i.1,
						"filename": a
					})
			})
			.collect();

		let msg_id = match self.socket
			.send_message(chat, text, subject, Some(serde_json::Value::Array(json_info)), photos_str)
			.await {

			Ok(id) => id,
			Err(err) => return Err(err.into())
		};

		for i in infos.iter_mut().zip(datas) {
			let mut data = i.1;
			let len = (i.0).0;
			let id = &(i.0).1;

			for idx in 0..=len {
				let chunk: Vec<u8> = data.drain(..std::cmp::min(data.len(), self.chunk_size)).collect();
				let base64_chunk = base64::encode(chunk);

				if let Err(err) = self.socket.attachment_data(id, &msg_id, idx, &base64_chunk).await {
					eprintln!("aaarrrggghh issue: {:?}", err);
				}
			}
		}

		Ok(())
	}
}
