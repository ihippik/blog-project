use gloo_net::http::Request;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use serde_wasm_bindgen as swb;
use wasm_bindgen::prelude::*;
use web_sys::{window, Storage};

const TOKEN_KEY: &str = "blog_token";

#[wasm_bindgen]
pub struct BlogApp {
    server_addr: String,
    token: Option<String>,
}

#[derive(Serialize)]
struct RegisterRequest {
    username: String,
    email: String,
    password: String,
}

#[derive(Serialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Serialize)]
struct PostPayload {
    title: String,
    content: String,
}

// Тип поста для десериализации ответа /api/protected/posts
#[derive(Debug, Serialize, Deserialize)]
struct Post {
    id: String,
    author_id: String,
    title: String,
    content: String,
    created_at: String,
}

fn to_js_error<E: std::fmt::Display>(e: E) -> JsValue {
    JsValue::from_str(&e.to_string())
}

fn storage() -> Result<Storage, JsValue> {
    let win = window().ok_or_else(|| JsValue::from_str("no window"))?;
    let storage = win
        .local_storage()
        .map_err(|e| JsValue::from_str(&format!("localStorage error: {:?}", e)))?
        .ok_or_else(|| JsValue::from_str("localStorage not available"))?;
    Ok(storage)
}

fn save_token_to_storage(token: &str) -> Result<(), JsValue> {
    let storage = storage()?;
    storage
        .set_item(TOKEN_KEY, token)
        .map_err(|e| JsValue::from_str(&format!("Failed to save token: {:?}", e)))
}

fn get_token_from_storage() -> Result<Option<String>, JsValue> {
    let storage = storage()?;
    let res = storage
        .get_item(TOKEN_KEY)
        .map_err(|e| JsValue::from_str(&format!("Failed to read token: {:?}", e)))?;
    Ok(res)
}

fn remove_token_from_storage() -> Result<(), JsValue> {
    let storage = storage()?;
    let _ = storage.remove_item(TOKEN_KEY);
    Ok(())
}

impl BlogApp {
    fn url(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.server_addr.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    async fn response_to_jsvalue(resp: gloo_net::http::Response) -> Result<JsValue, JsValue> {
        let status = resp.status();
        if !resp.ok() {
            let text = resp.text().await.unwrap_or_default();
            return Err(JsValue::from_str(&format!(
                "HTTP {}: {}",
                status, text
            )));
        }

        let text = resp.text().await.map_err(to_js_error)?;
        if text.trim().is_empty() {
            return Ok(JsValue::UNDEFINED);
        }

        let json: Value = serde_json::from_str(&text).map_err(to_js_error)?;
        swb::to_value(&json).map_err(to_js_error)
    }

    fn set_token(&mut self, token: &str) -> Result<(), JsValue> {
        self.token = Some(token.to_string());
        save_token_to_storage(token)
    }

    fn extract_and_store_token(
        &mut self,
        json: &Value,
    ) -> Result<(), JsValue> {
        if let Some(token) = json.get("access_token").and_then(|t| t.as_str()) {
            self.set_token(token)?;
        }
        Ok(())
    }

    fn get_current_token(&self) -> Result<Option<String>, JsValue> {
        if let Some(t) = &self.token {
            return Ok(Some(t.clone()));
        }
        get_token_from_storage()
    }
}

#[wasm_bindgen]
impl BlogApp {
    #[wasm_bindgen(constructor)]
    pub fn new(addr: String) -> BlogApp {
        let token = get_token_from_storage().unwrap_or(None);
        BlogApp {
            server_addr: addr,
            token,
        }
    }

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

    #[wasm_bindgen]
    pub fn logout(&mut self) -> Result<(), JsValue> {
        self.token = None;
        remove_token_from_storage()
    }

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

        // 👇 ключ: десериализуем в Vec<Post>
        let posts: Vec<Post> = serde_json::from_str(&text).map_err(to_js_error)?;
        // и уже его превращаем в JsValue — в JS это будет Array<{id, title, ...}>
        swb::to_value(&posts).map_err(to_js_error)
    }

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

    #[wasm_bindgen(js_name = "isAuthenticated")]
    pub fn is_authenticated(&self) -> Result<JsValue, JsValue> {
        let has = self.token.is_some()
            || get_token_from_storage().unwrap_or(None).is_some();
        Ok(JsValue::from_bool(has))
    }
}
