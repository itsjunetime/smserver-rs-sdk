use std::time::Duration;
use crate::{
	config::*,
	error::*,
	registration_type::*,
};

pub struct RestAPIClient {
	pub client: reqwest::Client,
	pub config: SDKConfig,
	pub authenticated: bool
}

impl RestAPIClient {
	pub fn new(config: SDKConfig) -> RestAPIClient {
		// these specific things are to make sure that the client can connect
		// with SMServer, since it uses a self-signed cert and normally connects
		// with an IP Address, not hostname

		let tls = native_tls::TlsConnector::builder()
			.use_sni(false)
			.danger_accept_invalid_certs(true)
			.danger_accept_invalid_hostnames(true)
			.build()
			.expect("Unable to build TlsConnector");

		let client = reqwest::Client::builder()
			.use_native_tls()
			.use_preconfigured_tls(tls)
			.connect_timeout(Duration::from_secs(config.timeout as u64))
			.build()
			.expect("Unable to build API Client");

		RestAPIClient {
			authenticated: false,
			config,
			client
		}
	}

	pub async fn get_url_string(&self, url: &str) -> anyhow::Result<String> {
		let response = self.client.get(url).send().await?;

		Ok(response.text().await.unwrap_or_else(|_| "".to_owned()))
	}

	pub async fn get_url_data(&self, url: &str) -> anyhow::Result<Vec<u8>> {
		let response = self.client.get(url).send().await?;

		Ok(response.bytes().await?.to_vec())
	}

	pub async fn authenticate(&mut self) -> anyhow::Result<bool> {
		// authenticate with SMServer so that we can make more requests later
		// without being denied
		let pass = format!("requests?password={}", self.config.password());
		let url = self.config.push_to_rest_url(pass);
		let res = self.get_url_string(&url).await?;

		Ok(res.parse().unwrap_or(false))
	}

	pub async fn check_auth(&mut self) -> anyhow::Result<()> {
		if self.config.use_rest && !self.authenticated {
			match self.authenticate().await? {
				true => self.authenticated = true,
				false => return Err(SDKError::UnAuthenticated.into()),
			}
		}
		
		Ok(())
	}

	pub async fn register_socket(
		&self,
		key: impl Into<String>,
		host_key: impl Into<String>,
		reg_type: RegistrationType
	) -> anyhow::Result<String> {
		let reg_str = match reg_type {
			RegistrationType::HostClient => "hostclient",
			RegistrationType::Lobby => "lobby"
		};

		let url = format!("register?key={}&host_key={}&reg_type={}",
			key.into(), host_key.into(), reg_str);

		let register_url = self.config.push_to_sock_url(url);

		self.get_url_string(&register_url).await
	}

	pub async fn remove_registration(
		&self,
		id: impl Into<String>,
		key: impl Into<String>,
		host_key: impl Into<String>
	) -> anyhow::Result<()> {
		let url = format!("remove?id={}&key={}&host_key={}",
			id.into(), key.into(), host_key.into());

		let remove_url = self.config.push_to_sock_url(url);

		self.get_url_string(&remove_url)
			.await
			.map(|_| ())
	}
}
