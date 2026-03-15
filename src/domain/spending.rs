use std::collections::HashMap;
use std::sync::Mutex;

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
    pub notes: Option<String>,
    pub edit_state: EditState,
}

pub struct DraftStore {
    drafts: Mutex<HashMap<i64, SpendingDraft>>,
}

impl DraftStore {
    pub fn new() -> Self {
        Self {
            drafts: Mutex::new(HashMap::new()),
        }
    }

    pub fn get(&self, telegram_id: i64) -> Option<SpendingDraft> {
        self.drafts.lock().unwrap().get(&telegram_id).cloned()
    }

    pub fn set(&self, telegram_id: i64, draft: SpendingDraft) {
        self.drafts.lock().unwrap().insert(telegram_id, draft);
    }

    pub fn update_state(&self, telegram_id: i64, state: EditState) -> bool {
        let mut drafts = self.drafts.lock().unwrap();
        if let Some(draft) = drafts.get_mut(&telegram_id) {
            draft.edit_state = state;
            true
        } else {
            false
        }
    }

    pub fn update_category(&self, telegram_id: i64, category_id: i64) {
        let mut drafts = self.drafts.lock().unwrap();
        if let Some(draft) = drafts.get_mut(&telegram_id) {
            draft.category_id = category_id;
            draft.edit_state = EditState::Summary;
        }
    }

    pub fn update_account(&self, telegram_id: i64, account_id: i64) {
        let mut drafts = self.drafts.lock().unwrap();
        if let Some(draft) = drafts.get_mut(&telegram_id) {
            draft.account_id = account_id;
            draft.edit_state = EditState::Summary;
        }
    }

    pub fn update_note(&self, telegram_id: i64, note: String) {
        let mut drafts = self.drafts.lock().unwrap();
        if let Some(draft) = drafts.get_mut(&telegram_id) {
            draft.notes = Some(note);
            draft.edit_state = EditState::Summary;
        }
    }

    pub fn remove(&self, telegram_id: i64) -> Option<SpendingDraft> {
        self.drafts.lock().unwrap().remove(&telegram_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_draft() -> SpendingDraft {
        SpendingDraft {
            amount: 15.50,
            category_id: 1,
            account_id: 1,
            reporter_user_id: 1,
            notes: None,
            edit_state: EditState::Summary,
        }
    }

    #[test]
    fn test_draft_store_basic() {
        let store = DraftStore::new();
        assert!(store.get(100).is_none());

        store.set(100, make_draft());
        let draft = store.get(100).unwrap();
        assert_eq!(draft.amount, 15.50);
        assert_eq!(draft.edit_state, EditState::Summary);
    }

    #[test]
    fn test_update_state() {
        let store = DraftStore::new();
        store.set(100, make_draft());

        assert!(store.update_state(100, EditState::ChoosingCategory));
        assert_eq!(
            store.get(100).unwrap().edit_state,
            EditState::ChoosingCategory
        );

        assert!(!store.update_state(999, EditState::Summary));
    }

    #[test]
    fn test_update_category() {
        let store = DraftStore::new();
        store.set(100, make_draft());
        store.update_state(100, EditState::ChoosingCategory);

        store.update_category(100, 5);
        let draft = store.get(100).unwrap();
        assert_eq!(draft.category_id, 5);
        assert_eq!(draft.edit_state, EditState::Summary);
    }

    #[test]
    fn test_update_account() {
        let store = DraftStore::new();
        store.set(100, make_draft());

        store.update_account(100, 3);
        let draft = store.get(100).unwrap();
        assert_eq!(draft.account_id, 3);
        assert_eq!(draft.edit_state, EditState::Summary);
    }

    #[test]
    fn test_update_note() {
        let store = DraftStore::new();
        store.set(100, make_draft());
        store.update_state(100, EditState::EnteringNote);

        store.update_note(100, "lunch".to_string());
        let draft = store.get(100).unwrap();
        assert_eq!(draft.notes.as_deref(), Some("lunch"));
        assert_eq!(draft.edit_state, EditState::Summary);
    }

    #[test]
    fn test_remove() {
        let store = DraftStore::new();
        store.set(100, make_draft());

        let removed = store.remove(100).unwrap();
        assert_eq!(removed.amount, 15.50);
        assert!(store.get(100).is_none());
    }

    #[test]
    fn test_new_amount_replaces_draft() {
        let store = DraftStore::new();
        store.set(100, make_draft());

        let mut new_draft = make_draft();
        new_draft.amount = 99.99;
        store.set(100, new_draft);

        assert_eq!(store.get(100).unwrap().amount, 99.99);
    }
}
