#[macro_export]
macro_rules! log{
	($msg:expr$(, $vars:expr)*) => {
		crate::config::SDKConfig::log(format!($msg$(, $vars)*));
	}
}

pub struct SDKConfig {
	pub rest_base_url: String,
	pub sock_base_url: String,
	pub password: String,
	pub timeout: usize, // in seconds
	pub chunk_size: usize, // in bytes
	pub use_rest: bool,
	pub secure: bool,
}

impl SDKConfig {
	pub fn default() -> SDKConfig {
		SDKConfig {
			rest_base_url: "".to_owned(),
			sock_base_url: "".to_owned(),
			password: "toor".to_owned(),
			timeout: 10,
			chunk_size: 51200,
			use_rest: true,
			secure: true,
		}
	}

	pub fn with_rest_url(mut self, url: impl Into<String>) -> Self {
		let full_url = url.into();

		if full_url.ends_with('/') {
			self.rest_base_url = full_url[..full_url.len() - 1].to_owned();
		} else {
			self.rest_base_url = full_url;
		}

		self
	}

	pub fn with_password(mut self, pass: impl Into<String>) -> Self {
		self.password = pass.into();
		self
	}

	pub fn with_sock_url(mut self, url: impl Into<String>) -> Self {
		let full_url = url.into();

		if full_url.ends_with('/') {
			self.sock_base_url = full_url;
		} else {
			self.sock_base_url = format!("{}/", full_url);
		}

		self
	}

	pub fn with_chunk_size(mut self, size: usize) -> Self {
		self.chunk_size = size;
		self
	}

	pub fn with_timeout(mut self, time: usize) -> Self {
		self.timeout = time;
		self
	}

	pub fn with_rest(mut self, rest: bool) -> Self {
		self.use_rest = rest;
		self
	}

	pub fn with_secure(mut self, sec: bool) -> Self {
		self.secure = sec;
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

	pub fn log(log_str: String) {
		use std::{
			fs::OpenOptions,
			io::Write
		};

		let mut file = OpenOptions::new()
			.create(true)
			.append(true)
			.open("log.log")
			.expect("Cannot open log file for writing");

		let _ = writeln!(file, "{}", log_str);
	}
}
