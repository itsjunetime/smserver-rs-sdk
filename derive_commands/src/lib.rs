extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::DeriveInput;
use std::vec::Vec;
use proc_macro2::{
	Ident,
	Span,
};

const TOTAL_STR: &str = "total";
const COLLECTED_STR: &str = "current";
const DATA_STR: &str = "data";

#[proc_macro_derive(Commands, attributes(parameters, data, command))]
pub fn commands_derive(input: TokenStream) -> TokenStream {
	let ast: DeriveInput = syn::parse(input).unwrap();

	// only allowed to be used on enums
	let name = &ast.ident;
	let variants = match &ast.data {
		syn::Data::Enum(enm) => {
			&enm.variants
		},
		_ => panic!("Please only use this on enums")
	};

	// use this config to track the parameters that were defined for each
	// API Command through the `command` attribute
	let mut config = CommandConfig {
		subdir: None,
		rest: true,
		socket: true,
		data_return: false,
		multipart: false,
		files_key: None,
		authenticate: true,
		return_type: None,
		no_main: false,
	};

	// the `${Command}Notification` structs that are being made
	let mut structs = Vec::new();
	// the functions that are being built to convert a SocketResponse
	// into each of the `${Command}Notification`s
	let mut impls = Vec::new();
	// the functions that are being built for the rest api to communicate
	let mut rest_fns = Vec::new();
	// the functions that will reside within the APIClient struct
	// to easily communicate with both rest api and socket
	let mut main_fns = Vec::new();

	let (matches, sock_fns) = variants.iter()
		.fold(
			(Vec::new(), Vec::new()),
			| (mut cmds, mut fns), var | {
				// this changes everything besides the subdirectory back
				// to its default, so that you don't have to set all the attributes
				// for every command
				config.reset();

				let ident = &var.ident;
				let id = var.ident.to_string();

				// parsed = the original name of the variant, but lowercase
				// with dashes inserted between the words.
				// .e.g GetChat => get-chat
				let parsed = id.chars()
					.enumerate()
					.fold(
						"".to_owned(),
						| mut s, (i, c) | {
							if c.is_uppercase() && i > 0 {
								s.push('-');
							}
							s.push(c.to_ascii_lowercase());
							s
					});

				// generates the command_string for this variant
				// e.g. `RequestCommand::GetChats => "get-chats".to_owned()`
				let gen = quote!{
					#name::#ident => #parsed.to_owned()
				};

				// this just changes the get-chats to get_chats so it can be used
				// as the name of a function
				let fn_name = parsed.replace("-", "_");
				let struct_name =
					format_ident!("{}Notification", ident.to_string());

				for i in var.attrs.iter() {
					// the name, either `parameters`, `data`, or `command`
					let path_name = i.path.segments[0].ident.to_string();

					// meta is simply a data structure which makes it easier
					// for us to build the fns and structs we want
					let meta = i.parse_meta()
						.expect("Unable to parse params as meta");

					match path_name.as_str() {
						// `parameters` defines the parameters that will be used
						// when sending to the socket/REST API
						"parameters" => if config.socket || config.rest {
							// get the function to send this to the socket, if
							// the command didn't forbid creating it
							if config.socket {
								let sock_fn =
									get_sock_fn(&meta, &fn_name, ident, name);
								fns.push(sock_fn);
							}

							// get the function to send this through the rest_api,
							// if the command didn't forbid creating it
							if config.rest {
								let rest_fn =
									get_rest_fn(&meta, &fn_name, &config);
								rest_fns.push(rest_fn);
							}


							if !config.no_main {
								let main_cmd = main_cmd(&meta, &fn_name, &config);
								main_fns.push(main_cmd);
							}
						},
						// `data` defines the parameters of the data that
						// will be sent from the socket as a notification
						"data" if config.socket => {
							// generate the struct and the function to turn a
							// SocketResponse into that struct
							let (struct_gen, impl_gen) =
								parse_data(
									meta, &fn_name, &struct_name, name, &config
								);
							structs.push(struct_gen);
							impls.push(impl_gen);
						},
						// `command` simply sets options for what the macro should
						// do with the `data` and `parameters` attributes
						"command" => {
							config.set_from_meta(&meta);
						},
						&_ => ()
					}
				}

				cmds.push(gen);
				(cmds, fns)
		});

	// build the final thing
	let gen = quote!{
		use crate::error::*;

		impl #name {
			pub fn command_string(&self) -> String {
				match self {
					#(#matches),*
				}
			}
		}

		// we impl it for crate::...::SocketHandler since it's the one
		// with the send_command function and the SplitSink to send it with.

		// Also this custom derive is only intended to be used
		// with smserver-rs-sdk, so there's no need to make it agnostic or whatever
		impl crate::socket::SocketHandler {
			#(#sock_fns)*
		}

		#(#structs)*

		impl crate::socket::SocketResponse {
			#(#impls)*
		}

		impl crate::rest_api::RestAPIClient {
			#(#rest_fns)*
		}

		impl crate::api::APIClient {
			#(#main_fns)*
		}
	};

	gen.into()
}

fn get_sock_fn(
	params: &syn::Meta, fn_name: &str, ident: &Ident, name: &Ident
) -> proc_macro2::TokenStream {
	// `list` should be a list of all the key-value pairs in the
	// parameter parentheses
	let nvs = get_name_val_list(&params);

	let (values, inserts) = nvs.iter()
		.fold(
			(Vec::new(), Vec::new()),
			| (mut vals, mut inserts), (path, param_type) | {
				// path is the `chat` in `chat = "me"`
				let path_str = path.to_string();

				let fn_quote = if param_type.starts_with("Option<") {
					// do the `if let Some(_) = _` so that we only insert
					// this value to the map if it is included
					quote!{
						if let Some(val) = #path {
							map.insert(#path_str.to_owned(),
								serde_json::Value::from(val));
						}
					}
				} else {
					// if it's not optional, just insert it and set the
					// type as not optional.
					quote!{
						map.insert(#path_str.to_owned(),
							serde_json::Value::from(#path));
					}
				};

				// since the type may be like `Option<String>`, we can't do it as
				// an ident, we have to do it as a TokenStream
				let type_stream: proc_macro2::TokenStream = param_type.parse()
					.expect(&format!("Unable to parse {} as TStream", param_type));

				let val_quote = quote!{
					#path: #type_stream
				};

				vals.push(val_quote);
				inserts.push(fn_quote);

				(vals, inserts)
		});

	// fn_ident is an ident for the functionname
	let fn_ident = Ident::new(&fn_name, Span::call_site());

	// the function itself
	quote!{
		pub async fn #fn_ident(
			&mut self,
			#(#values),*
		) -> ::std::result::Result<
			::std::string::String,
			tokio_tungstenite::tungstenite::Error
		> {
			let mut map = serde_json::Map::new();
			#(#inserts);*

			self.send_command(#name::#ident, map.into()).await
		}
	}
}

fn get_rest_fn(
	params: &syn::Meta,
	fn_name: &str,
	config: &CommandConfig
) -> proc_macro2::TokenStream {
	// this builds the function for the REST API to communicate with whatever part
	// of the API is specified in the parameters

	// this contains the name-value pairs of the attribute, easily parseable
	let mut nvs = get_name_val_list(&params);
	let fn_ident = Ident::new(&fn_name, Span::call_site());

	/*let auth = if config.authenticate {
		quote!{ self.check_auth().await?; }
	} else {
		quote!{}
	};*/

	// the functions for a multipart form and a GET request look dramatically
	// different, so we have to split here to cater to each
	if config.multipart {
		// if they specified a key to be used as the files in the multipart...
		if let Some(ref fs_key) = config.files_key {
			// check if it exists in the parameters
			let pos = nvs.iter().position(|p| p.0.to_string() == *fs_key);
			// if it does, remove it so that we don't set it as another parameter
			// for the generated function
			if let Some(p) = pos {
				nvs.remove(p);
			}
		}

		// the quote that actually creates the form. If they don't specify a key
		// for the files, generate an empty form
		let form_quote = if let Some(ref fs) = config.files_key {
			quote!{
				let mut form = if let Some(fil) = files {
					fil.iter().fold(
						reqwest::multipart::Form::new(),
						| form, file | {
							if let Ok(data) = std::fs::read(file) {
								let part = reqwest::multipart::Part::bytes(data);
								form.part(#fs, part)
							} else {
								form
							}
					})
				} else {
					// also accomodate for if they just don't pass in any files
					// when calling the function
					reqwest::multipart::Form::new()
				};
			}
		} else {
			quote!{
				let mut form = reqwest::multipart::Form::new();
			}
		};

		// get the blocks of code that will add values to the form and
		// the blocks that define which type each parameter needs to be
		//let (add_quotes, mut values) = kp_vec.iter().fold(
		let (add_quotes, mut values) = nvs.iter().fold(
			(Vec::new(), Vec::new()),
			| (mut quos, mut vals), (key, typ) | {
				// make sure the types are presented as a string
				//let key = pair.0;
				let key_str = key.to_string();

				// once again, do special parsing to accomodate for
				// Options since I use them so much
				let push = if typ.starts_with("Option<") {
					quote!{
						if let Some(val) = #key {
							form = form.text(#key_str, val);
						}
					}
				} else {
					quote!{
						form = form.text(#key_str, #key);
					}
				};

				quos.push(push);

				let type_ident: proc_macro2::TokenStream =
					typ.parse().unwrap();

				// make the code that shows the type
				let type_quote = quote!{
					#key: #type_ident
				};

				vals.push(type_quote);
				(quos, vals)
		});

		// also make the parameter that defines the files, if we didn't already
		if config.files_key.is_some() {
			values.push(
				quote!{
					files: std::option::Option<std::vec::Vec<std::string::String>>
				}
			);
		}

		// and make the code that generates the string to send it to
		let req_str = if let Some(ref sbd) = config.subdir {
			let subdir = sbd.to_owned();
			quote!{
				let req_str = self.config.push_to_rest_url(#subdir);
			}
		} else {
			quote!{
				let req_str = self.config.push_to_rest_url("send");
			}
		};

		// the result!
		quote!{
			pub async fn #fn_ident(
				&mut self,
				#(#values),*
			) -> anyhow::Result<()> {
				self.check_auth().await?;

				#req_str

				#form_quote

				#(#add_quotes);*

				self.client.post(&req_str)
					.multipart(form)
					.send()
					.await?;

				Ok(())
			}
		}

	} else {
		// this part is for sending a GET request using a URL Query string

		// get the code that states the type of each parameter
		// the the code that implements adding them into the query string
		let (values, queries) = nvs.iter()
			.fold(
				(Vec::new(), Vec::new()),
				| (mut vals, mut qs), (path, param_type) | {

					let path_str = path.to_string();

					// this makes the code that actually adds them to the
					// query string.
					// Once again, special parsing for options
					let fn_quote = if param_type.starts_with("Option<") {
						quote!{
							query_string = format!("{}&{}{}",
								query_string,
								#path_str,
								if let Some(v) = #path {
									format!("={}", v.to_string())
								} else {
									"".to_owned()
								});
						}
					} else {
						quote!{
							query_string = format!("{}&{}{}",
								query_string, #path_str,
								if #path.to_string().len() > 0 {
									format!("={}", #path.to_string())
								} else {
									"".to_string()
								});
						}
					};

					// get the type, since it can't be ident, etc
					let type_stream: proc_macro2::TokenStream = param_type.parse()
						.expect(&format!("Can't parse {} as TStream", param_type));

					let val_quote = quote!{
						#path: #type_stream
					};

					vals.push(val_quote);
					qs.push(fn_quote);

					(vals, qs)
			});

		let fn_ident = Ident::new(&fn_name, Span::call_site());

		// this function could return data or json, so change the return type and
		// function to call based on which one we want to use
		let (get_quote, ret_type) = if config.data_return {
			let get_quote = quote!{
				self.get_url_data(&query_string).await
			};

			let ret_type = quote!{ Vec<u8> };

			(get_quote, ret_type)
		} else if let Some(typ) = &config.return_type {
			let get_quote = quote!{
				let res = self.get_url_string(&query_string).await?;
				let val = serde_json::Value::from_str(&res)?;
				Ok(serde_json::from_value(val)?)
			};

			let ret_type = typ.parse().unwrap();

			(get_quote, ret_type)
		} else {
			let get_quote = quote!{
				self.get_url_string(&query_string).await?;
				Ok(())
			};
			let ret_type = quote!{ () };

			(get_quote, ret_type)
		};

		let subdir = match &config.subdir {
			Some(sub) => sub.to_owned(),
			None => "".to_owned()
		};

		// final result!
		quote!{
			pub async fn #fn_ident(
				&mut self,
				#(#values),*
			) -> anyhow::Result<#ret_type> {
				self.check_auth().await?;

				let mut query_string = self.config.push_to_rest_url(#subdir);
				#(#queries);*

				#get_quote
			}
		}
	}
}

fn parse_data(
	data: syn::Meta, fn_name: &str, struct_name: &Ident, enum_name: &Ident, config: &CommandConfig
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
	let mut paths = Vec::new();

	let idents = vec![
		Ident::new(DATA_STR, Span::call_site()),
		Ident::new(COLLECTED_STR, Span::call_site()),
		Ident::new(TOTAL_STR, Span::call_site()),
	];

	let nvs = if config.data_return {
		vec![
			(&idents[0], "String".to_owned()),
			(&idents[1], "u32".to_owned()),
			(&idents[2], "u32".to_owned())
		]
	} else {
		get_name_val_list(&data)
	};

	let (values, serials) = nvs.iter()
		.fold(
			(Vec::new(), Vec::new()),
			| (mut vals, mut sers), (path, param_type) | {

				let type_stream: proc_macro2::TokenStream = param_type.parse()
					.expect(&format!("Unable to parse {} as TStream", param_type));

				let type_quote = quote!{
					#path: #type_stream
				};

				let val_quote = quote!{ pub #type_quote };

				let pstr = path.to_string();

				let ser_quote = quote!{
					let #type_quote = serde_json::from_value(
						self.data.get_mut(#pstr)
							.unwrap_or(&mut serde_json::Value::Null).take()
					)?;
				};

				vals.push(val_quote);
				sers.push(ser_quote);

				paths.push(path.to_owned());

				(vals, sers)
		});

	let struct_quote = quote!{
		pub struct #struct_name {
			pub id: String,
			pub command: #enum_name,
			#(#values),*
		}
	};

	let fn_name_ident = format_ident!("{}_data", fn_name);

	let impl_quote = quote!{
		pub fn #fn_name_ident(
			mut self
		) -> Result<#struct_name, serde_json::error::Error> {
			let id = self.id;
			let command = self.command;
			#(#serials)*
			Ok(#struct_name {
				id,
				command,
				#(#paths),*
			})
		}
	};

	(struct_quote, impl_quote)
}

fn main_cmd(
	meta: &syn::Meta, fn_name: &str, config: &CommandConfig
) -> proc_macro2::TokenStream {
	let nvs = get_name_val_list(meta);

	let types = nvs.iter().map(|(i, v)| {
		let typ: proc_macro2::TokenStream = v.parse().unwrap();
		quote!{ #i: #typ }
	});

	let names: Vec<&Ident> = nvs.iter().map(|p| p.0).collect();

	let fn_ident = Ident::new(fn_name, Span::call_site());

	let rest_section = match config.rest {
		true => quote!{
			if self.rest_client.authenticated {
				return self.rest_client.#fn_ident(#(#names),*).await;
			} else {
				return Err(SDKError::UnAuthenticated.into());
			}
		},
		_ => quote!{
			return Err(SDKError::ConfigBlocked.into())
		}
	};

	let (response, res_type):
		(proc_macro2::TokenStream, proc_macro2::TokenStream) =
	match config.return_type.is_some() || config.data_return {
		true => (
			quote!{
				match serde_json::from_value(msg.data) {
					Ok(val) => Ok(val),
					Err(err) => Err(err.into())
				}
			},
			match config.data_return {
				true => quote!{ Vec<u8> },
				_ => config.return_type.as_ref().unwrap().parse().unwrap(),
			}
		),
		_ => (
			"Ok(())".parse().unwrap(),
			"()".parse().unwrap()
		)
	};

	let receiving_section = match config.data_return {
		true => quote!{
			let mut current = 0;
			let mut total_data: Vec<u8> = Vec::new();

			while let Ok(msg) = receiver.recv() {
				let json = match msg.data.as_object() {
					Some(val) => val,
					None => {
						current = 0;
						break;
					}
				};

				let mut new_data = match base64::decode(
					json[#DATA_STR].as_str().unwrap()
				) {
					Ok(val) => val,
					Err(err) => return Err(err.into())
				};

				total_data.append(&mut new_data);

				current += 1;
				let total = match json[#TOTAL_STR].as_i64() {
					Some(val) => val,
					None => {
						current = 0;
						break;
					}
				};

				if current == total {
					break;
				}
			}

			match current {
				0 => Err(SDKError::MangledReceive.into()),
				_ => Ok(total_data)
			}
		},
		_ => quote!{
			if let Ok(msg) = receiver.recv() {
				return #response;
			}

			return Err(SDKError::MangledReceive.into());
		}
	};

	let sock_section = match config.socket {
		true => if config.data_return || config.return_type.is_some() {
			quote!{
				let id = match self.socket.#fn_ident(#(#names),*).await {
					Ok(id) => id,
					Err(err) => return Err(err.into())
				};
				let (sender, receiver) = std::sync::mpsc::sync_channel(0);

				if let Ok(mut msgs) = self.sock_msgs.write() {
					msgs.insert(id, sender);
				}

				#receiving_section
			}
		} else {
			quote!{
				match self.socket.#fn_ident(#(#names),*).await {
					Ok(_) => Ok(()),
					Err(err) => Err(err.into())
				}
			}
		},
		_ => quote!{
			Err(SDKError::ConfigBlocked.into())
		}
	};

	quote!{
		pub async fn #fn_ident(
			&mut self,
			#(#types),*
		) -> anyhow::Result<#res_type> {
			if self.uses_rest {
				#rest_section
			}

			#sock_section
		}
	}
}

fn get_name_val_list(data: &syn::Meta) -> Vec<(&Ident, String)> {
	get_name_vals(data).iter()
		.map(|p| {
			let val = if let syn::Lit::Str(v)= p.1 {
				v.value()
			} else {
				panic!("Please only use on strings");
			};

			(p.0, val)
		})
		.collect()
}

fn get_name_vals(data: &syn::Meta) -> Vec<(&Ident, &syn::Lit)> {
	if let syn::Meta::List(list) = data {
		list.nested.iter()
			.fold(
				Vec::new(),
				| mut vals, nv | {
					if let syn::NestedMeta::Meta(syn::Meta::NameValue(pair)) = nv {
						let id = &pair.path.segments.first()
							.expect("Can't get first segment :(")
							.ident;

						vals.push((id, &pair.lit));
					}

					vals
			})
	} else {
		panic!("Only send meta lists to this function");
	}
}

struct CommandConfig {
	pub subdir: Option<String>,
	pub rest: bool,
	pub socket: bool,
	pub data_return: bool,
	pub multipart: bool,
	pub files_key: Option<String>,
	pub authenticate: bool,
	pub return_type: Option<String>,
	pub no_main: bool,
}

impl CommandConfig {
	pub fn reset(&mut self) {
		self.rest = true;
		self.socket = true;
		self.data_return = false;
		self.multipart = false;
		self.files_key = None;
		self.authenticate = true;
		self.return_type = None;
		self.no_main = false;
	}

	pub fn set_from_meta(&mut self, meta: &syn::Meta) {
		let nvs = get_name_vals(&meta);

		for (path_str, lit) in nvs.iter() {
			match path_str.to_string().as_str() {
				"subdir" => if let syn::Lit::Str(dir) = lit {
					self.subdir = Some(dir.value());
				},
				"rest" => if let syn::Lit::Bool(rest) = lit {
					self.rest = rest.value();
				},
				"socket" => if let syn::Lit::Bool(sock) = lit {
					self.socket = sock.value();
				},
				"data_return" => if let syn::Lit::Bool(data) = lit {
					self.data_return = data.value();
				},
				"multipart" => if let syn::Lit::Bool(mult) = lit {
					self.multipart = mult.value();
				},
				"files" => if let syn::Lit::Str(fil) = lit {
					self.files_key = Some(fil.value());
				},
				"authenticate" => if let syn::Lit::Bool(auth) = lit {
					self.authenticate = auth.value();
				},
				"return_type" => if let syn::Lit::Str(res) = lit {
					self.return_type = Some(res.value())
				},
				"no_main" => if let syn::Lit::Bool(main) = lit {
					self.no_main = main.value()
				},
				&_ => ()
			}
		}
	}
}
