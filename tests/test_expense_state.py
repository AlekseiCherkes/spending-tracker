"""Tests for the expense state management module."""

from datetime import datetime

from spending_tracker.expense_state import DraftExpense, ExpenseStateManager


class TestDraftExpense:
    """Test the DraftExpense class."""

    def test_draft_expense_creation_minimal(self):
        """Test creating a draft expense with minimal required fields."""
        draft = DraftExpense(amount=25.50, telegram_id=123456)

        assert draft.amount == 25.50
        assert draft.telegram_id == 123456
        assert draft.category_id is None
        assert draft.account_id is None
        assert draft.notes is None
        assert draft.timestamp is None
        assert draft.message_id is None

    def test_draft_expense_creation_complete(self):
        """Test creating a draft expense with all fields."""
        timestamp = datetime.now()
        draft = DraftExpense(
            amount=100.00,
            telegram_id=789012,
            category_id=1,
            account_id=2,
            notes="Test expense",
            timestamp=timestamp,
            message_id=123,
        )

        assert draft.amount == 100.00
        assert draft.telegram_id == 789012
        assert draft.category_id == 1
        assert draft.account_id == 2
        assert draft.notes == "Test expense"
        assert draft.timestamp == timestamp
        assert draft.message_id == 123

    def test_is_complete_true(self):
        """Test is_complete returns True when all required fields are set."""
        draft = DraftExpense(amount=25.50, telegram_id=123456, category_id=1, account_id=2)

        assert draft.is_complete() is True

    def test_is_complete_false_missing_category(self):
        """Test is_complete returns False when category_id is missing."""
        draft = DraftExpense(amount=25.50, telegram_id=123456, account_id=2)

        assert draft.is_complete() is False

    def test_is_complete_false_missing_account(self):
        """Test is_complete returns False when account_id is missing."""
        draft = DraftExpense(amount=25.50, telegram_id=123456, category_id=1)

        assert draft.is_complete() is False

    def test_is_complete_false_missing_both(self):
        """Test is_complete returns False when both category_id and account_id are missing."""
        draft = DraftExpense(amount=25.50, telegram_id=123456)

        assert draft.is_complete() is False

    def test_is_complete_false_zero_amount(self):
        """Test is_complete returns False when amount is zero."""
        draft = DraftExpense(amount=0.0, telegram_id=123456, category_id=1, account_id=2)

        assert draft.is_complete() is False

    def test_to_dict_minimal(self):
        """Test converting draft to dictionary with minimal fields."""
        draft = DraftExpense(amount=25.50, telegram_id=123456, category_id=1, account_id=2)

        result = draft.to_dict()

        # Check that all required fields are present
        assert result["amount"] == 25.50
        assert result["category_id"] == 1
        assert result["account_id"] == 2
        assert result["reporter_id"] == 123456
        assert result["notes"] is None
        assert isinstance(result["timestamp"], datetime)

    def test_to_dict_complete(self):
        """Test converting draft to dictionary with all fields."""
        timestamp = datetime.now()
        draft = DraftExpense(
            amount=100.00,
            telegram_id=789012,
            category_id=1,
            account_id=2,
            notes="Test expense",
            timestamp=timestamp,
            message_id=123,
        )

        result = draft.to_dict()

        expected = {
            "amount": 100.00,
            "category_id": 1,
            "account_id": 2,
            "reporter_id": 789012,
            "notes": "Test expense",
            "timestamp": timestamp,
        }

        assert result == expected

    def test_to_dict_includes_reporter_id_excludes_others(self):
        """Test that to_dict includes reporter_id but excludes telegram_id and message_id."""
        draft = DraftExpense(amount=25.50, telegram_id=123456, category_id=1, account_id=2, message_id=999)

        result = draft.to_dict()

        assert "reporter_id" in result
        assert result["reporter_id"] == 123456
        assert "telegram_id" not in result
        assert "message_id" not in result

    def test_to_dict_sets_timestamp_if_none(self):
        """Test that to_dict sets timestamp if None."""
        draft = DraftExpense(amount=25.50, telegram_id=123456, category_id=1, account_id=2, timestamp=None)

        result = draft.to_dict()

        assert isinstance(result["timestamp"], datetime)
        assert "reporter_id" in result
        assert result["reporter_id"] == 123456


class TestExpenseStateManager:
    """Test the ExpenseStateManager class."""

    def setup_method(self):
        """Set up test state manager for each test."""
        self.manager = ExpenseStateManager()

    def test_create_draft(self):
        """Test creating a draft expense."""
        draft = self.manager.create_draft(123456, 25.50)

        assert isinstance(draft, DraftExpense)
        assert draft.amount == 25.50
        assert draft.telegram_id == 123456
        assert self.manager.has_draft(123456)

        retrieved_draft = self.manager.get_draft(123456)
        assert retrieved_draft.amount == 25.50
        assert retrieved_draft.telegram_id == 123456

    def test_get_draft_exists(self):
        """Test getting an existing draft."""
        self.manager.create_draft(789012, 50.00)
        retrieved_draft = self.manager.get_draft(789012)

        assert retrieved_draft is not None
        assert retrieved_draft.amount == 50.00
        assert retrieved_draft.telegram_id == 789012

    def test_get_draft_not_exists(self):
        """Test getting a non-existent draft."""
        draft = self.manager.get_draft(999999)

        assert draft is None

    def test_update_draft(self):
        """Test updating an existing draft."""
        self.manager.create_draft(123456, 25.50)

        # Update the draft
        updated_draft = self.manager.update_draft(
            123456, amount=35.75, category_id=2, account_id=3, notes="Updated notes"
        )

        # Verify the update
        assert updated_draft is not None
        assert updated_draft.amount == 35.75
        assert updated_draft.category_id == 2
        assert updated_draft.account_id == 3
        assert updated_draft.notes == "Updated notes"

        # Verify it's the same object in manager
        retrieved_draft = self.manager.get_draft(123456)
        assert retrieved_draft.amount == 35.75
        assert retrieved_draft.category_id == 2
        assert retrieved_draft.account_id == 3
        assert retrieved_draft.notes == "Updated notes"

    def test_update_draft_not_exists(self):
        """Test updating a non-existent draft."""
        result = self.manager.update_draft(999999, amount=25.50)

        assert result is None
        assert not self.manager.has_draft(999999)

    def test_update_draft_invalid_field(self):
        """Test updating draft with invalid field is ignored."""
        self.manager.create_draft(123456, 25.50)

        updated_draft = self.manager.update_draft(123456, amount=35.75, invalid_field="should be ignored")

        assert updated_draft is not None
        assert updated_draft.amount == 35.75
        assert not hasattr(updated_draft, "invalid_field")

    def test_remove_draft(self):
        """Test removing an existing draft."""
        self.manager.create_draft(123456, 25.50)
        assert self.manager.has_draft(123456)

        result = self.manager.remove_draft(123456)
        assert result is True
        assert not self.manager.has_draft(123456)

    def test_remove_draft_not_exists(self):
        """Test removing a non-existent draft."""
        result = self.manager.remove_draft(999999)
        assert result is False

    def test_has_draft_true(self):
        """Test has_draft returns True for existing draft."""
        self.manager.create_draft(123456, 25.50)
        assert self.manager.has_draft(123456) is True

    def test_has_draft_false(self):
        """Test has_draft returns False for non-existent draft."""
        assert self.manager.has_draft(999999) is False

    def test_multiple_drafts_different_users(self):
        """Test managing multiple drafts for different users."""
        self.manager.create_draft(111111, 25.50)
        self.manager.create_draft(222222, 75.00)

        # Both drafts should exist
        assert self.manager.has_draft(111111)
        assert self.manager.has_draft(222222)

        # Each user should get their own draft
        retrieved_draft1 = self.manager.get_draft(111111)
        retrieved_draft2 = self.manager.get_draft(222222)

        assert retrieved_draft1.amount == 25.50
        assert retrieved_draft2.amount == 75.00

        # Removing one shouldn't affect the other
        self.manager.remove_draft(111111)
        assert not self.manager.has_draft(111111)
        assert self.manager.has_draft(222222)

    def test_overwrite_existing_draft(self):
        """Test that creating a draft overwrites existing one."""
        self.manager.create_draft(123456, 25.50)
        self.manager.create_draft(123456, 50.00)

        # Should have the new draft
        retrieved_draft = self.manager.get_draft(123456)
        assert retrieved_draft.amount == 50.00

    def test_get_all_drafts(self):
        """Test getting all drafts."""
        self.manager.create_draft(111111, 25.50)
        self.manager.create_draft(222222, 75.00)

        all_drafts = self.manager.get_all_drafts()

        assert len(all_drafts) == 2
        assert 111111 in all_drafts
        assert 222222 in all_drafts
        assert all_drafts[111111].amount == 25.50
        assert all_drafts[222222].amount == 75.00

    def test_clear_all_drafts(self):
        """Test clearing all drafts."""
        self.manager.create_draft(111111, 25.50)
        self.manager.create_draft(222222, 50.00)

        # Clear all drafts
        self.manager._drafts.clear()

        # Both drafts should be gone
        assert not self.manager.has_draft(111111)
        assert not self.manager.has_draft(222222)

    def test_manager_separate_instances(self):
        """Test that different manager instances have separate state."""
        manager1 = ExpenseStateManager()
        manager2 = ExpenseStateManager()

        manager1.create_draft(123456, 25.50)

        # manager2 should NOT see the draft (separate instances)
        assert not manager2.has_draft(123456)
        assert manager2.get_draft(123456) is None
