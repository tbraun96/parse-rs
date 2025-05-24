// src/client.rs

use crate::error::ParseError;
use crate::user::{LoginRequest, ParseUser, PasswordResetRequest, SignupRequest, SignupResponse};
use reqwest::{Client, Method, Response}; 
use serde::{de::DeserializeOwned, Serialize};
use url::Url;

// Helper struct for deserializing Parse API error responses
#[derive(serde::Deserialize, Debug)]
struct ParseApiErrorResponse {
    code: u16,
    error: String, 
}

#[derive(Debug, Clone)]
pub struct ParseClient {
    pub(crate) server_url: Url,
    pub(crate) app_id: String,
    pub(crate) javascript_key: Option<String>,
    pub(crate) rest_api_key: Option<String>,
    pub(crate) master_key: Option<String>,
    pub(crate) http_client: Client,
    pub(crate) session_token: Option<String>,
}

impl ParseClient {
    pub fn new(
        server_url_str: &str,
        app_id: &str,
        javascript_key: Option<&str>,
        rest_api_key: Option<&str>,
        master_key: Option<&str>,
    ) -> Result<Self, ParseError> {
        let mut server_url = Url::parse(server_url_str)?;
        if !server_url.path().ends_with('/') {
            server_url.set_path(&format!("{}/", server_url.path()));
        }

        let http_client = Client::new();

        Ok(Self {
            server_url,
            app_id: app_id.to_string(),
            javascript_key: javascript_key.map(String::from),
            rest_api_key: rest_api_key.map(String::from),
            master_key: master_key.map(String::from),
            http_client,
            session_token: None,
        })
    }

    fn _set_session_token(&mut self, token: Option<String>) {
        self.session_token = token;
    }

    pub fn get_session_token(&self) -> Option<&str> {
        self.session_token.as_deref()
    }

    async fn _request<S: Serialize + ?Sized>(
        &self,
        method: Method,
        endpoint: &str,
        body: Option<&S>,
        override_session_token: Option<Option<&str>>,
    ) -> Result<Response, ParseError> {
        let request_url = self.server_url.join(endpoint)?;

        let mut request_builder = self.http_client.request(method.clone(), request_url);

        request_builder = request_builder.header("X-Parse-Application-Id", &self.app_id);

        if let Some(master_key) = &self.master_key {
            request_builder = request_builder.header("X-Parse-Master-Key", master_key);
        } else if let Some(rest_key) = &self.rest_api_key {
            request_builder = request_builder.header("X-Parse-REST-API-Key", rest_key);
        } 

        let mut token_to_send: Option<String> = None; 
        let mut send_token_header = true;

        match override_session_token {
            Some(Some(token_str)) => token_to_send = Some(token_str.to_string()),
            Some(None) => send_token_header = false, 
            None => token_to_send = self.session_token.clone(), 
        }

        if send_token_header {
            if let Some(ref token) = token_to_send {
                if !(endpoint == "users" && method == Method::POST) && 
                   !(endpoint == "login" && method == Method::POST) &&
                   !(endpoint == "requestPasswordReset" && method == Method::POST) { 
                    request_builder = request_builder.header("X-Parse-Session-Token", token);
                }
            }
        }

        if body.is_some() {
            request_builder = request_builder.header("Content-Type", "application/json");
            request_builder = request_builder.json(body.unwrap()); 
        }

        let response = request_builder.send().await?;

        if response.status().is_success() {
            Ok(response)
        } else {
            let status = response.status();
            let error_response_text = response.text().await?;
            
            if let Ok(parsed_error) = serde_json::from_str::<ParseApiErrorResponse>(&error_response_text) {
                Err(ParseError::ApiError {
                    code: parsed_error.code,
                    message: parsed_error.error,
                })
            } else {
                Err(ParseError::ApiError {
                    code: status.as_u16(),
                    message: format!("HTTP Error: {}. Response: {}", status, error_response_text),
                })
            }
        }
    }

    pub(crate) async fn get<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T, ParseError> {
        let response = self._request(Method::GET, endpoint, None::<&()>, None).await?;
        response.json::<T>().await.map_err(ParseError::Network)
    }

    pub(crate) async fn post<S: Serialize + ?Sized, T: DeserializeOwned>(
        &self,
        endpoint: &str,
        body: &S,
    ) -> Result<T, ParseError> {
        let response = self._request(Method::POST, endpoint, Some(body), None).await?;
        response.json::<T>().await.map_err(ParseError::Network)
    }

    pub(crate) async fn put<S: Serialize + ?Sized, T: DeserializeOwned>(
        &self,
        endpoint: &str,
        body: &S,
    ) -> Result<T, ParseError> {
        let response = self._request(Method::PUT, endpoint, Some(body), None).await?;
        response.json::<T>().await.map_err(ParseError::Network)
    }

    pub(crate) async fn delete(&self, endpoint: &str) -> Result<(), ParseError> {
        self._request(Method::DELETE, endpoint, None::<&()>, None).await?;
        Ok(())
    }

    // --- User Authentication Methods ---

    pub async fn signup<'a>(
        &mut self, 
        username: &'a str, 
        password: &'a str, 
        email: Option<&'a str>
    ) -> Result<SignupResponse, ParseError> {
        let request_body = SignupRequest {
            username,
            password,
            email,
        };
        
        let signup_data: SignupResponse = self.post("users", &request_body).await?;
        
        self.session_token = Some(signup_data.session_token.clone());

        Ok(signup_data)
    }

    pub async fn login<'a>(
        &mut self, 
        username: &'a str, 
        password: &'a str
    ) -> Result<ParseUser, ParseError> {
        let request_body = LoginRequest {
            username,
            password,
        };
        let user: ParseUser = self.post("login", &request_body).await?;
        if let Some(token) = &user.session_token {
            self._set_session_token(Some(token.clone()));
        }
        Ok(user)
    }

    pub fn is_authenticated(&self) -> bool {
        self.session_token.is_some()
    }

    pub async fn logout(&mut self) -> Result<(), ParseError> {
        if self.session_token.is_none() {
            return Ok(());
        }
        self._request(Method::POST, "logout", None::<&()>, None).await?;
        self._set_session_token(None); 
        Ok(())
    }

    pub async fn current_user(&self) -> Result<Option<ParseUser>, ParseError> {
        if !self.is_authenticated() {
            return Ok(None);
        }
        match self.get::<ParseUser>("users/me").await {
            Ok(user) => Ok(Some(user)),
            Err(ParseError::ApiError { code, .. }) if code == 209 => {
                Ok(None)
            }
            Err(e) => Err(e),
        }
    }

    pub async fn r#become(&mut self, new_session_token: &str) -> Result<ParseUser, ParseError> {
        let original_token = self.session_token.clone();
        self._set_session_token(Some(new_session_token.to_string()));

        let response = self._request(
            Method::GET, 
            "users/me", 
            None::<&()>, 
            Some(Some(new_session_token))
        ).await?;

        match response.json::<ParseUser>().await {
            Ok(user) => {
                Ok(user)
            }
            Err(reqwest_error) => {
                self._set_session_token(original_token);
                Err(ParseError::Network(reqwest_error))
            }
        }
    }

    pub async fn request_password_reset<'a>(&self, email: &'a str) -> Result<(), ParseError> {
        let request_body = PasswordResetRequest { email };
        self._request(Method::POST, "requestPasswordReset", Some(&request_body), Some(None)).await?;
        Ok(())
    }
}
