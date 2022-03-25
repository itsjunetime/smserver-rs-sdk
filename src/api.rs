use std::sync::Arc;
use dashmap::DashMap;
use crate::{
	rest_api::RestAPIClient,
	socket::*,
	config::*,
};
use serde_json::json;

pub struct APIClient {
	pub rest_client: RestAPIClient,
	pub socket: SocketHandler,
	pub sock_msgs: Arc<DashMap<String, crossbeam_channel::Sender<SocketResponse>>>,
	pub uses_rest: bool,
	pub chunk_size: usize,
}

impl APIClient {
	// so when this struct is initialized, it needs to pass a clone of the
	// sock_msgs hashmap & notif_rec receiver into the spawn_receiver function
	// of the socket handler.
	//
	// Then, every time the socket handler receives a new message,
	// it automatically grabs the crossbeam_channel::Sender that relates to the id of the msg.
	// It sends the socket response through the sender, which is received by
	// the receiver who is awaiting a message.
	//
	// If there is no sender, it just sends the data through an crossbeam_channel::Sender that
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
		let (sender, receiver) = crossbeam_channel::unbounded();

		self.sock_msgs.insert(id, sender);

		Ok(receiver.recv()?.do_command_data())

		if let Ok(msg) = receiver.recv() {
			let parsed_res: DoCommandResponse = msg.do_command_data();
			Ok(parsed_res)
			// or Ok(msg.do_command_data())
		}

		Err(SDKError::MangledReceive.into())
	}
	*/

	pub async fn new(
		config: SDKConfig, sender: crossbeam_channel::Sender<SocketResponse>
	) -> anyhow::Result<APIClient> {
		let chunk_size = config.chunk_size;
		let uses_rest = config.use_rest;
		let base_url = config.sock_base_url.to_owned();

		// for now, we create the RestAPIClient even if we're not using rest.
		// Should probably fix that up sooner or later.

		let mut rest_client = RestAPIClient::new(config);
		let sock_msgs = Arc::new(DashMap::new());

		// parse the url since we need that for settings up the socket
		let url = url::Url::parse(&base_url)?;

		if uses_rest {
			rest_client.check_auth().await?;
		}

		let socket = SocketHandler::new(url, sender, sock_msgs.clone()).await?;

		Ok(APIClient{
			rest_client,
			socket,
			sock_msgs,
			uses_rest,
			chunk_size
		})
	}

	pub async fn authenticate(&mut self) -> anyhow::Result<bool> {
		self.rest_client.authenticate().await
	}

	// I custom-wrote a function for this since it's so complicated to send it
	// over a socket
	pub async fn send_message(
		&mut self,
		chat: String,
		text: Option<String>,
		subject: Option<String>,
		attachments: Option<Vec<String>>,
		photos: Option<Vec<String>>,
	) -> anyhow::Result<()> {
		// This is how I submit the photos, just use a colon to separate them
		// (since I'm fairly certain you aren't allowed to have a colon in
		// a filename in xnu)
		let photos_str = photos.map(|p| p.join(":"));

		if self.uses_rest {
			// fairly straightforward for this
			return self.rest_client
				.send_message(chat, text, subject, photos_str, attachments)
				.await;
		}

		// datas: Actual data of the files
		// infos: (number of messages needed, attachment's id)
		let (datas, mut infos): (Vec<String>, Vec<(u32, String)>) =
		match attachments {
			None => (Vec::new(), Vec::new()),
			Some(ref files) => files.iter().fold(
				(Vec::new(), Vec::new()), | (mut d, mut i), f | {
					// read the data of the file
					let bin = match std::fs::read(f) {
						Ok(bin) => base64::encode(bin),
						Err(_) => String::new(),
					};

					let len = bin.len();

					// add the data to the data vector
					d.push(bin);

					// create the id and get the total size
					let id = uuid::Uuid::new_v4().to_string();

					// divide size by chunk size to figure out how many messages
					// will be needed to send this completely over
					let len = (len as f64 / self.chunk_size as f64).ceil() as u32;

					i.push((len, id));

					(d, i)
				}),
		};

		// create the JSON info for each attachment. This is what'll be sent with
		// the first message to tell the host what to expect when I do send
		// the attachment data
		let json_info = infos.iter()
			.zip(attachments.unwrap_or_default())
			.map(
				| (i, a) | {
					json!({
						"size": i.0,
						"id": i.1,
						"filename": a.split('/').last().unwrap_or(&a)
					})
			})
			.collect();

		// send the original message
		let msg_id = match self.socket.send_message(
			chat,
			text,
			subject,
			Some(serde_json::Value::Array(json_info)),
			photos_str
		).await {
			Ok(id) => id,
			Err(err) => return Err(err.into())
		};

		// iterate over all the attachments
		for i in infos.iter_mut().zip(datas) {
			let mut data = i.1;
			let len = (i.0).0;
			let id = &(i.0).1;

			// iterate over how many messages will be needed to send the data
			for idx in 0..=(len-1) {
				// get the chunk. Drain it from the b64 vector so that we end up
				// with an empty vector once we've sent the data
				let chunk: String = data.drain(
					..std::cmp::min(data.len(), self.chunk_size)
				).collect();

				// the chunk is already base64-encoded, so just
				// send the data for this chunk
				if let Err(_err) = self.socket.attachment_data(
					id, &msg_id, idx, &chunk
				).await {
					// Do something, I guess?? Well, maybe just suffer.
				}
			}
		}

		Ok(())
	}
}
