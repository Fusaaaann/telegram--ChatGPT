use serde::{Serialize, Deserialize};
use rand::distributions::Alphanumeric;
use rand::{Rng, thread_rng};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
struct User {
    chat_id: String,
    #[serde(skip_serializing, skip_deserializing)]
    mutex: tokio::sync::RwLock<()>,
    conversation: Vec<String>,
}

impl User {
    LIMIT = std::env::var("CONVERSATION_MEMORY_LIMIT").unwrap_or(50);
    fn save_message(role:&str,content:&str) {
        // get write lock of User
        // create copy of role, content
        // push content to end of conversation
    }
    fn save_conversation() {
        // save whole conversation as 
    }
}