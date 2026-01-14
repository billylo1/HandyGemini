use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct ConversationMessage {
    pub role: String, // "user" or "model"
    pub text: String,
}

pub struct GeminiConversationManager {
    conversation: Arc<Mutex<Vec<ConversationMessage>>>,
}

impl GeminiConversationManager {
    pub fn new() -> Self {
        Self {
            conversation: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn add_user_message(&self, text: String) {
        let mut conv = self.conversation.lock().unwrap();
        conv.push(ConversationMessage {
            role: "user".to_string(),
            text,
        });
    }

    pub fn add_model_message(&self, text: String) {
        let mut conv = self.conversation.lock().unwrap();
        conv.push(ConversationMessage {
            role: "model".to_string(),
            text,
        });
    }

    pub fn get_history(&self) -> Vec<ConversationMessage> {
        self.conversation.lock().unwrap().clone()
    }

    pub fn clear(&self) {
        let mut conv = self.conversation.lock().unwrap();
        conv.clear();
    }
}
