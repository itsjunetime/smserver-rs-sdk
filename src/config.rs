pub struct SDKConfig {
	pub rest_base_url: String,
	pub sock_base_url: String,
	pub password: String,
	pub chunk_size: u32 // in bytes
}

impl SDKConfig {
	pub fn default() -> SDKConfig {
		SDKConfig {
			rest_base_url: "".to_owned(),
			sock_base_url: "".to_owned(),
			password: "toor".to_owned(),
			chunk_size: 51200
		}
	}

	pub fn with_rest_url(&mut self, url: impl Into<String>) -> &mut SDKConfig {
		self.rest_base_url = url.into();
		self
	}

	pub fn with_password(&mut self, pass: impl Into<String>) -> &mut SDKConfig {
		self.password = pass.into();
		self
	}

	pub fn with_sock_url(&mut self, url: impl Into<String>) -> &mut SDKConfig {
		self.sock_base_url = url.into();
		self
	}

	pub fn with_chunk_size(&mut self, size: u32) -> &mut SDKConfig {
		self.chunk_size = size;
		self
	}

	pub fn password(&self) -> &str {
		&self.password
	}

	pub fn push_to_rest_url(&self, url: impl Into<String>) -> String {
		format!("{}/{}", self.rest_base_url, url.into())
	}

	pub fn push_to_sock_url(&self, url: impl Into<String>) -> String {
		format!("{}/{}", self.sock_base_url, url.into())
	}
}
