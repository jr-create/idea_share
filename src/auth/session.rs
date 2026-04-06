use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Session {
    pub user_id: i64,
    pub username: String,
    pub token: String,
    pub csrf_token: String,
}

#[derive(Debug, Clone)]
pub struct SessionStore {
    store: Arc<Mutex<HashMap<String, Session>>>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn create_session(&self, user_id: i64, username: &str) -> (String, String) {
        let token = Uuid::new_v4().to_string();
        let csrf_token = Uuid::new_v4().to_string();
        let session = Session {
            user_id,
            username: username.to_string(),
            token: token.clone(),
            csrf_token: csrf_token.clone(),
        };
        
        let mut store = self.store.lock().await;
        store.insert(token.clone(), session);
        (token, csrf_token)
    }

    pub async fn get_session(&self, token: &str) -> Option<Session> {
        let store = self.store.lock().await;
        store.get(token).cloned()
    }

    pub async fn remove_session(&self, token: &str) {
        let mut store = self.store.lock().await;
        store.remove(token);
    }
}