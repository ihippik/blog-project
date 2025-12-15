use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_wasm_bindgen as swb;
use wasm_bindgen::prelude::*;
use web_sys::{window, Storage};

/// Key used to store the JWT token in browser storage.
const TOKEN_KEY: &str = "blog_token";

/// WASM client for interacting with the Blog backend.
///
/// Exposed to JavaScript via `wasm-bindgen`.
#[wasm_bindgen]
pub struct BlogApp {
    server_addr: String,
    token: Option<String>,
}

/// User registration request payload.
#[derive(Serialize)]
struct RegisterRequest {
    username: String,
    email: String,
    password: String,
}

/// User login request payload.
#[derive(Serialize)]
struct LoginRequest {
    email: String,
    password: String,
}

/// Post creation and update payload.
#[derive(Serialize)]
struct PostPayload {
    title: String,
    content: String,
}

/// Post model used for deserializing API responses.
#[derive(Debug, Serialize, Deserialize)]
struct Post {
    id: String,
    author_id: String,
    title: String,
    content: String,
    created_at: String,
}

/// Converts an error into a JavaScript value.
fn to_js_error<E: std::fmt::Display>(e: E) -> JsValue {
    JsValue::from_str(&e.to_string())
}

/// Returns browser local storage.
fn storage() -> Result<Storage, JsValue> {
    let win = window().ok_or_else(|| JsValue::from_str("no window"))?;
    let storage = win
        .local_storage()
        .map_err(|e| JsValue::from_str(&format!("localStorage error: {:?}", e)))?
        .ok_or_else(|| JsValue::from_str("localStorage not available"))?;
    Ok(storage)
}

/// Saves a JWT token to browser storage.
fn save_token_to_storage(token: &str) -> Result<(), JsValue> {
    let storage = storage()?;
    storage
        .set_item(TOKEN_KEY, token)
        .map_err(|e| JsValue::from_str(&format!("Failed to save token: {:?}", e)))
}

/// Loads a JWT token from browser storage.
fn get_token_from_storage() -> Result<Option<String>, JsValue> {
    let storage = storage()?;
    let res = storage
        .get_item(TOKEN_KEY)
        .map_err(|e| JsValue::from_str(&format!("Failed to read token: {:?}", e)))?;
    Ok(res)
}

/// Removes the JWT token from browser storage.
fn remove_token_from_storage() -> Result<(), JsValue> {
    let storage = storage()?;
    let _ = storage.remove_item(TOKEN_KEY);
    Ok(())
}

impl BlogApp {
    /// Builds a full API URL from a relative path.
    fn url(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.server_addr.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    /// Converts an HTTP response into a `JsValue`.
    async fn response_to_jsvalue(
        resp: gloo_net::http::Response,
    ) -> Result<JsValue, JsValue> {
        let status = resp.status();
        if !resp.ok() {
            let text = resp.text().await.unwrap_or_default();
            return Err(JsValue::from_str(&format!("HTTP {}: {}", status, text)));
        }

        let text = resp.text().await.map_err(to_js_error)?;
        if text.trim().is_empty() {
            return Ok(JsValue::UNDEFINED);
        }

        let json: Value = serde_json::from_str(&text).map_err(to_js_error)?;
        swb::to_value(&json).map_err(to_js_error)
    }

    /// Stores the JWT token in memory and browser storage.
    fn set_token(&mut self, token: &str) -> Result<(), JsValue> {
        self.token = Some(token.to_string());
        save_token_to_storage(token)
    }

    /// Extracts and stores a JWT token from a JSON response.
    fn extract_and_store_token(&mut self, json: &Value) -> Result<(), JsValue> {
        if let Some(token) = json.get("access_token").and_then(|t| t.as_str()) {
            self.set_token(token)?;
        }
        Ok(())
    }

    /// Returns the currently active JWT token, if any.
    fn get_current_token(&self) -> Result<Option<String>, JsValue> {
        if let Some(t) = &self.token {
            return Ok(Some(t.clone()));
        }
        get_token_from_storage()
    }
}

#[wasm_bindgen]
impl BlogApp {
    /// Creates a new Blog WASM client.
    #[wasm_bindgen(constructor)]
    pub fn new(addr: String) -> BlogApp {
        let token = get_token_from_storage().unwrap_or(None);
        BlogApp {
            server_addr: addr,
            token,
        }
    }

    /// Registers a new user.
    #[wasm_bindgen]
    pub async fn register(
        &mut self,
        username: String,
        email: String,
        password: String,
    ) -> Result<JsValue, JsValue> {
        let body = RegisterRequest {
            username,
            email,
            password,
        };

        let url = self.url("/register");

        let resp = Request::post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .map_err(to_js_error)?
            .send()
            .await
            .map_err(to_js_error)?;

        let status = resp.status();
        let text = resp.text().await.map_err(to_js_error)?;
        if status < 200 || status >= 300 {
            return Err(JsValue::from_str(&format!(
                "Register failed ({}): {}",
                status, text
            )));
        }

        let json: Value = serde_json::from_str(&text).map_err(to_js_error)?;
        self.extract_and_store_token(&json)?;
        swb::to_value(&json).map_err(to_js_error)
    }

    /// Authenticates a user and stores the JWT token.
    #[wasm_bindgen]
    pub async fn login(
        &mut self,
        email: String,
        password: String,
    ) -> Result<JsValue, JsValue> {
        let body = LoginRequest { email, password };
        let url = self.url("/api/public/auth/login");

        let resp = Request::post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .map_err(to_js_error)?
            .send()
            .await
            .map_err(to_js_error)?;

        let status = resp.status();
        let text = resp.text().await.map_err(to_js_error)?;
        if status < 200 || status >= 300 {
            return Err(JsValue::from_str(&format!(
                "Login failed ({}): {}",
                status, text
            )));
        }

        let json: Value = serde_json::from_str(&text).map_err(to_js_error)?;
        self.extract_and_store_token(&json)?;
        swb::to_value(&json).map_err(to_js_error)
    }

    /// Logs out the current user.
    #[wasm_bindgen]
    pub fn logout(&mut self) -> Result<(), JsValue> {
        self.token = None;
        remove_token_from_storage()
    }

    /// Loads posts of the authenticated user.
    #[wasm_bindgen(js_name = "loadPosts")]
    pub async fn load_posts(&self) -> Result<JsValue, JsValue> {
        let token = self
            .get_current_token()?
            .ok_or_else(|| JsValue::from_str("Not authenticated"))?;

        let url = self.url("/api/protected/posts");

        let resp = Request::get(&url)
            .header("Authorization", &format!("Bearer {}", token))
            .send()
            .await
            .map_err(to_js_error)?;

        let status = resp.status();
        let text = resp.text().await.map_err(to_js_error)?;
        if status < 200 || status >= 300 {
            return Err(JsValue::from_str(&format!(
                "Load posts failed ({}): {}",
                status, text
            )));
        }

        let posts: Vec<Post> = serde_json::from_str(&text).map_err(to_js_error)?;
        swb::to_value(&posts).map_err(to_js_error)
    }

    /// Creates a new post.
    #[wasm_bindgen(js_name = "createPost")]
    pub async fn create_post(
        &self,
        title: String,
        content: String,
    ) -> Result<JsValue, JsValue> {
        let token = self
            .get_current_token()?
            .ok_or_else(|| JsValue::from_str("Not authenticated"))?;

        let body = PostPayload { title, content };
        let url = self.url("/api/protected/posts");

        let resp = Request::post(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", &format!("Bearer {}", token))
            .json(&body)
            .map_err(to_js_error)?
            .send()
            .await
            .map_err(to_js_error)?;

        BlogApp::response_to_jsvalue(resp).await
    }

    /// Updates an existing post.
    #[wasm_bindgen(js_name = "updatePost")]
    pub async fn update_post(
        &self,
        id: String,
        title: String,
        content: String,
    ) -> Result<JsValue, JsValue> {
        let token = self
            .get_current_token()?
            .ok_or_else(|| JsValue::from_str("Not authenticated"))?;

        let body = PostPayload { title, content };
        let url = self.url(&format!("/api/protected/posts/{}", id));

        let resp = Request::put(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", &format!("Bearer {}", token))
            .json(&body)
            .map_err(to_js_error)?
            .send()
            .await
            .map_err(to_js_error)?;

        BlogApp::response_to_jsvalue(resp).await
    }

    /// Deletes a post by its ID.
    #[wasm_bindgen(js_name = "deletePost")]
    pub async fn delete_post(&self, id: String) -> Result<JsValue, JsValue> {
        let token = self
            .get_current_token()?
            .ok_or_else(|| JsValue::from_str("Not authenticated"))?;

        let url = self.url(&format!("/api/protected/posts/{}", id));

        let resp = Request::delete(&url)
            .header("Authorization", &format!("Bearer {}", token))
            .send()
            .await
            .map_err(to_js_error)?;

        BlogApp::response_to_jsvalue(resp).await
    }

    /// Returns whether the user is authenticated.
    #[wasm_bindgen(js_name = "isAuthenticated")]
    pub fn is_authenticated(&self) -> Result<JsValue, JsValue> {
        let has =
            self.token.is_some() || get_token_from_storage().unwrap_or(None).is_some();
        Ok(JsValue::from_bool(has))
    }
}
