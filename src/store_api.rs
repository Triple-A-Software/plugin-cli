use keyring::Entry;
use reqwest::blocking::multipart;
use serde::Deserialize;
use serde_json::json;

use crate::{utils::SoftPanic, PluginMetadata};

pub struct LoggedIn;
pub struct NotLoggedIn;

pub struct Client<State = NotLoggedIn> {
    reqwest: reqwest::blocking::Client,
    remote: String,
    cred_entry: Entry,
    state: std::marker::PhantomData<State>,
}

impl Client<NotLoggedIn> {
    pub fn login(self, email: &str, pw: Option<String>) -> Result<Client<LoggedIn>, ClientError> {
        let cred_entry = Entry::new("plugin-cli", email).unwrap();
        if pw.is_none() {
            cred_entry
                .get_password()
                .map_err(|_| ClientError::LoginFailed)?;
            return Ok(Client {
                reqwest: self.reqwest.clone(),
                remote: self.remote.clone(),
                cred_entry,
                state: std::marker::PhantomData::<LoggedIn>,
            });
        }
        cred_entry.delete_password().unwrap();
        let body = json!({
            "email": email,
            "password": pw
        });
        let token = self
            .reqwest
            .post(format!("{}/api/rest/auth/login", self.remote))
            .json(&body)
            .header("User-Agent", "plugin-cli")
            .send()
            .expect("Could not login");
        if !token.status().is_success() {
            return Err(ClientError::LoginFailed);
        }
        let token = token.text().expect("Could not read login response");
        if token.is_empty() {
            return Err(ClientError::LoginFailed);
        }
        cred_entry.set_password(&token).unwrap();
        Ok(Client {
            reqwest: self.reqwest,
            remote: self.remote,
            cred_entry,
            state: std::marker::PhantomData::<LoggedIn>,
        })
    }
}

impl Client<LoggedIn> {
    pub fn publish_plugin(&self, metadata: &PluginMetadata) -> Result<(), ClientError> {
        let name = metadata.name.clone();
        let form = multipart::Form::new()
            .text("id", name)
            .text("version", metadata.version.to_string())
            .file("file", metadata.archive_name())
            .expect("Failed to open archive file");
        let token = self
            .cred_entry
            .get_password()
            .map_err(|_| ClientError::NotLoggedIn)?;
        let res = self
            .reqwest
            .post(format!("{}/api/rest/plugin/publish", self.remote))
            .multipart(form)
            .header("User-Agent", "plugin-cli")
            .header("Cookie", format!("plugin-store-session={}", token))
            .send()
            .map_err(|e| ClientError::PublishFailed(e.into()))?;
        let parsed: StoreApiResponse =
            serde_json::from_str(&res.text().soft_expect("Could not read response"))
                .soft_expect("Could not parse response");

        if let Some(error) = parsed.error {
            if error == "unauthorized" {
                let _ = self.cred_entry.delete_password();
                Err(ClientError::InvalidCredentials)
            } else {
                Err(ClientError::PublishFailed(anyhow::Error::msg(error)))
            }
        } else if let Some(success) = parsed.success {
            if success {
                Ok(())
            } else {
                Err(ClientError::PublishFailed(anyhow::Error::msg(
                    "Unknown error",
                )))
            }
        } else {
            Err(ClientError::Other)
        }
    }

    pub fn logout(self) -> Result<Client<NotLoggedIn>, ClientError> {
        self.cred_entry
            .delete_password()
            .map_err(|_| ClientError::NotLoggedIn)?;
        Ok(Client {
            reqwest: self.reqwest,
            remote: self.remote,
            cred_entry: self.cred_entry,
            state: Default::default(),
        })
    }
}

impl Client {
    pub fn new(remote: String, email: &str) -> Self {
        Client {
            reqwest: reqwest::blocking::Client::new(),
            remote,
            cred_entry: Entry::new("plugin-cli", email)
                .ok()
                .soft_expect("Could not create entry"),
            state: Default::default(),
        }
    }
}

impl<State> Client<State> {
    pub fn is_logged_in(&self) -> bool {
        self.cred_entry.get_password().is_ok()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("Login failed")]
    LoginFailed,

    #[error("Not logged in")]
    NotLoggedIn,

    #[error("Invalid credentials or token expired")]
    InvalidCredentials,

    #[error("Publish plugin failed: {0}")]
    PublishFailed(#[from] anyhow::Error),

    #[error("Other error")]
    Other,
}

#[derive(Debug, Deserialize)]
struct StoreApiResponse {
    success: Option<bool>,
    error: Option<String>,
}
