use std::collections::HashMap;
use std::sync::Mutex;

/// Key for drafts: (chat_id, message_id) of the bot's summary reply.
pub type DraftKey = (i64, i32);

#[derive(Debug, Clone, PartialEq)]
pub enum EditState {
    Summary,
    ChoosingCategory,
    ChoosingAccount,
    EnteringNote,
}

#[derive(Debug, Clone)]
pub struct SpendingDraft {
    pub amount: f64,
    pub category_id: i64,
    pub account_id: i64,
    pub reporter_user_id: i64,
    pub telegram_id: i64,
    pub notes: Option<String>,
    pub edit_state: EditState,
}

pub struct DraftStore {
    drafts: Mutex<HashMap<DraftKey, SpendingDraft>>,
}

impl DraftStore {
    pub fn new() -> Self {
        Self {
            drafts: Mutex::new(HashMap::new()),
        }
    }

    pub fn get(&self, key: DraftKey) -> Option<SpendingDraft> {
        self.drafts.lock().unwrap().get(&key).cloned()
    }

    pub fn set(&self, key: DraftKey, draft: SpendingDraft) {
        self.drafts.lock().unwrap().insert(key, draft);
    }

    pub fn update_state(&self, key: DraftKey, state: EditState) -> bool {
        let mut drafts = self.drafts.lock().unwrap();
        if let Some(draft) = drafts.get_mut(&key) {
            draft.edit_state = state;
            true
        } else {
            false
        }
    }

    pub fn update_category(&self, key: DraftKey, category_id: i64) {
        let mut drafts = self.drafts.lock().unwrap();
        if let Some(draft) = drafts.get_mut(&key) {
            draft.category_id = category_id;
            draft.edit_state = EditState::Summary;
        }
    }

    pub fn update_account(&self, key: DraftKey, account_id: i64) {
        let mut drafts = self.drafts.lock().unwrap();
        if let Some(draft) = drafts.get_mut(&key) {
            draft.account_id = account_id;
            draft.edit_state = EditState::Summary;
        }
    }

    pub fn update_note(&self, key: DraftKey, note: String) {
        let mut drafts = self.drafts.lock().unwrap();
        if let Some(draft) = drafts.get_mut(&key) {
            draft.notes = Some(note);
            draft.edit_state = EditState::Summary;
        }
    }

    pub fn remove(&self, key: DraftKey) -> Option<SpendingDraft> {
        self.drafts.lock().unwrap().remove(&key)
    }

    /// Find a draft in a given state for a specific user. Returns the key if found.
    pub fn find_by_state(&self, telegram_id: i64, state: EditState) -> Option<DraftKey> {
        self.drafts
            .lock()
            .unwrap()
            .iter()
            .find(|(_, d)| d.telegram_id == telegram_id && d.edit_state == state)
            .map(|(k, _)| *k)
    }

    /// Reset a user's draft from EnteringNote back to Summary (e.g. when a new amount arrives).
    pub fn cancel_note_entry(&self, telegram_id: i64) {
        let mut drafts = self.drafts.lock().unwrap();
        for draft in drafts.values_mut() {
            if draft.telegram_id == telegram_id && draft.edit_state == EditState::EnteringNote {
                draft.edit_state = EditState::Summary;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const USER_TG: i64 = 42;

    fn make_draft() -> SpendingDraft {
        SpendingDraft {
            amount: 15.50,
            category_id: 1,
            account_id: 1,
            reporter_user_id: 1,
            telegram_id: USER_TG,
            notes: None,
            edit_state: EditState::Summary,
        }
    }

    fn key(msg_id: i32) -> DraftKey {
        (100, msg_id) // chat_id=100
    }

    #[test]
    fn test_draft_store_basic() {
        let store = DraftStore::new();
        assert!(store.get(key(1)).is_none());

        store.set(key(1), make_draft());
        let draft = store.get(key(1)).unwrap();
        assert_eq!(draft.amount, 15.50);
        assert_eq!(draft.edit_state, EditState::Summary);
    }

    #[test]
    fn test_update_state() {
        let store = DraftStore::new();
        store.set(key(1), make_draft());

        assert!(store.update_state(key(1), EditState::ChoosingCategory));
        assert_eq!(
            store.get(key(1)).unwrap().edit_state,
            EditState::ChoosingCategory
        );

        assert!(!store.update_state(key(999), EditState::Summary));
    }

    #[test]
    fn test_update_category() {
        let store = DraftStore::new();
        store.set(key(1), make_draft());
        store.update_state(key(1), EditState::ChoosingCategory);

        store.update_category(key(1), 5);
        let draft = store.get(key(1)).unwrap();
        assert_eq!(draft.category_id, 5);
        assert_eq!(draft.edit_state, EditState::Summary);
    }

    #[test]
    fn test_update_account() {
        let store = DraftStore::new();
        store.set(key(1), make_draft());

        store.update_account(key(1), 3);
        let draft = store.get(key(1)).unwrap();
        assert_eq!(draft.account_id, 3);
        assert_eq!(draft.edit_state, EditState::Summary);
    }

    #[test]
    fn test_update_note() {
        let store = DraftStore::new();
        store.set(key(1), make_draft());
        store.update_state(key(1), EditState::EnteringNote);

        store.update_note(key(1), "lunch".to_string());
        let draft = store.get(key(1)).unwrap();
        assert_eq!(draft.notes.as_deref(), Some("lunch"));
        assert_eq!(draft.edit_state, EditState::Summary);
    }

    #[test]
    fn test_remove() {
        let store = DraftStore::new();
        store.set(key(1), make_draft());

        let removed = store.remove(key(1)).unwrap();
        assert_eq!(removed.amount, 15.50);
        assert!(store.get(key(1)).is_none());
    }

    #[test]
    fn test_multiple_drafts_coexist() {
        let store = DraftStore::new();

        let mut draft1 = make_draft();
        draft1.amount = 50.0;
        store.set(key(1), draft1);

        let mut draft2 = make_draft();
        draft2.amount = 100.0;
        store.set(key(2), draft2);

        assert_eq!(store.get(key(1)).unwrap().amount, 50.0);
        assert_eq!(store.get(key(2)).unwrap().amount, 100.0);
    }

    #[test]
    fn test_find_by_state() {
        let store = DraftStore::new();
        store.set(key(1), make_draft());
        store.set(key(2), make_draft());

        store.update_state(key(2), EditState::EnteringNote);

        assert_eq!(
            store.find_by_state(USER_TG, EditState::EnteringNote),
            Some(key(2))
        );
        assert_eq!(
            store.find_by_state(USER_TG, EditState::ChoosingCategory),
            None
        );
        // Different user — not found
        assert_eq!(store.find_by_state(999, EditState::EnteringNote), None);
    }

    #[test]
    fn test_cancel_note_entry() {
        let store = DraftStore::new();
        store.set(key(1), make_draft());
        store.update_state(key(1), EditState::EnteringNote);

        store.cancel_note_entry(USER_TG);
        assert_eq!(store.get(key(1)).unwrap().edit_state, EditState::Summary);
    }
}
