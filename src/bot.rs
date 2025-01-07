use std::collections::HashMap;

struct TransactionKey {
    telegram_user_id: u64,
    message_id: u64,
}

struct SpendingTrackerBot {
    transactions: HashMap<TransactionKey, TransactionState>
}

enum TransactionState {
    Initial,
    AccountEditing,
    CategoryEditing,
}

impl SpendingTrackerBot {
    pub fn new() -> Self {
        Self {
            transactions: HashMap::new()
        }
    }

    pub fn handle_message(&mut self, bot: teloxide::Bot, message: teloxide::types::Message) {
    }

    pub fn handle_callback_query(&mut self, bot: teloxide::Bot, callback_query: teloxide::types::CallbackQuery) {
    }
}