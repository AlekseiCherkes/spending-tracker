"""
Expense State Management
Manages pending expenses in memory before saving to database.
"""

from dataclasses import dataclass
from datetime import datetime
from typing import Any, Dict, Optional


@dataclass
class DraftExpense:
    """Represents a draft expense being composed by user."""

    # Required fields
    amount: float
    telegram_id: int

    # Optional fields with defaults
    category_id: Optional[int] = None
    account_id: Optional[int] = None
    notes: Optional[str] = None
    timestamp: Optional[datetime] = None

    # UI state
    message_id: Optional[int] = None  # For editing inline keyboards

    def is_complete(self) -> bool:
        """Check if expense has all required fields to be saved."""
        return self.amount > 0 and self.category_id is not None and self.account_id is not None

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for database saving."""
        return {
            "amount": self.amount,
            "category_id": self.category_id,
            "account_id": self.account_id,
            "notes": self.notes,
            "timestamp": self.timestamp or datetime.now(),
        }


class ExpenseStateManager:
    """Manages draft expenses in memory."""

    def __init__(self) -> None:
        # Map telegram_id -> DraftExpense
        self._drafts: Dict[int, DraftExpense] = {}

    def create_draft(self, telegram_id: int, amount: float) -> DraftExpense:
        """Create a new draft expense."""
        draft = DraftExpense(amount=amount, telegram_id=telegram_id, timestamp=datetime.now())
        self._drafts[telegram_id] = draft
        return draft

    def get_draft(self, telegram_id: int) -> Optional[DraftExpense]:
        """Get draft expense for user."""
        return self._drafts.get(telegram_id)

    def update_draft(self, telegram_id: int, **kwargs: Any) -> Optional[DraftExpense]:
        """Update draft expense fields."""
        draft = self._drafts.get(telegram_id)
        if draft:
            for key, value in kwargs.items():
                if hasattr(draft, key):
                    setattr(draft, key, value)
        return draft

    def remove_draft(self, telegram_id: int) -> bool:
        """Remove draft expense."""
        return self._drafts.pop(telegram_id, None) is not None

    def has_draft(self, telegram_id: int) -> bool:
        """Check if user has a draft expense."""
        return telegram_id in self._drafts

    def get_all_drafts(self) -> Dict[int, DraftExpense]:
        """Get all draft expenses (for debugging)."""
        return self._drafts.copy()


# Global state manager instance
expense_state_manager = ExpenseStateManager()
