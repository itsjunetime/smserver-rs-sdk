use thiserror::Error;

#[derive(Error, Debug)]
pub enum SDKError {
	#[error("Failed to authenticate")]
	UnAuthenticated,
	#[error("MPSC Receiver ran into an error while trying to receive")]
	MangledReceive,
	#[error("This function is not allowed by the current SDK Configuration")]
	ConfigBlocked,
	#[error("The data json was sent in an improper format")]
	ImproperDataFormat
}
