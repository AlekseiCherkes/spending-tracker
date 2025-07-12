"""Tests for the Data Access Layer."""

import os
import sqlite3
import tempfile
from datetime import datetime, timedelta

import pytest

from spending_tracker.dal import SpendingTrackerDAL


class TestSpendingTrackerDAL:
    """Test the SpendingTrackerDAL class."""

    def setup_method(self):
        """Set up test database for each test."""
        # Create a temporary database file for testing
        self.db_fd, self.db_path = tempfile.mkstemp(suffix=".db")
        os.close(self.db_fd)  # Close the file descriptor
        self.dal = SpendingTrackerDAL(self.db_path)

    def teardown_method(self):
        """Clean up after each test."""
        # Remove the temporary database file
        if os.path.exists(self.db_path):
            os.unlink(self.db_path)

    def test_create_user(self):
        """Test creating a new user."""
        user_id = self.dal.create_user("John Doe", 123456)

        assert user_id is not None
        assert user_id > 0

        # Verify user was created
        user = self.dal.get_user_by_id(user_id)
        assert user is not None
        assert user["name"] == "John Doe"
        assert user["telegram_id"] == 123456

    def test_create_user_duplicate_telegram_id(self):
        """Test creating a user with duplicate telegram_id raises error."""
        self.dal.create_user("John Doe", 123456)

        with pytest.raises(sqlite3.IntegrityError):
            self.dal.create_user("Jane Doe", 123456)

    def test_get_user_by_id(self):
        """Test retrieving a user by ID."""
        user_id = self.dal.create_user("John Doe", 123456)

        user = self.dal.get_user_by_id(user_id)
        assert user is not None
        assert user["id"] == user_id
        assert user["name"] == "John Doe"
        assert user["telegram_id"] == 123456

    def test_get_user_by_id_not_found(self):
        """Test retrieving a non-existent user by ID."""
        user = self.dal.get_user_by_id(999)
        assert user is None

    def test_get_user_by_telegram_id(self):
        """Test retrieving a user by Telegram ID."""
        user_id = self.dal.create_user("John Doe", 123456)

        user = self.dal.get_user_by_telegram_id(123456)
        assert user is not None
        assert user["id"] == user_id
        assert user["name"] == "John Doe"
        assert user["telegram_id"] == 123456

    def test_get_user_by_telegram_id_not_found(self):
        """Test retrieving a non-existent user by Telegram ID."""
        user = self.dal.get_user_by_telegram_id(999999)
        assert user is None

    def test_get_all_users(self):
        """Test retrieving all users."""
        # Initially empty
        users = self.dal.get_all_users()
        assert len(users) == 0

        # Add some users
        self.dal.create_user("John Doe", 123456)
        self.dal.create_user("Jane Smith", 789012)

        users = self.dal.get_all_users()
        assert len(users) == 2
        assert users[0]["name"] == "John Doe"
        assert users[1]["name"] == "Jane Smith"

    def test_update_user(self):
        """Test updating a user's name."""
        user_id = self.dal.create_user("John Doe", 123456)

        # Update the user
        result = self.dal.update_user(user_id, "John Smith")
        assert result is True

        # Verify the update
        user = self.dal.get_user_by_id(user_id)
        assert user["name"] == "John Smith"
        assert user["telegram_id"] == 123456  # Should not change

    def test_update_user_not_found(self):
        """Test updating a non-existent user."""
        result = self.dal.update_user(999, "New Name")
        assert result is False

    def test_delete_user(self):
        """Test deleting a user."""
        user_id = self.dal.create_user("John Doe", 123456)

        # Delete the user
        result = self.dal.delete_user(user_id)
        assert result is True

        # Verify the user is gone
        user = self.dal.get_user_by_id(user_id)
        assert user is None

    def test_delete_user_not_found(self):
        """Test deleting a non-existent user."""
        result = self.dal.delete_user(999)
        assert result is False

    def test_user_exists(self):
        """Test checking if a user exists."""
        # User doesn't exist yet
        assert self.dal.user_exists(123456) is False

        # Create user
        self.dal.create_user("John Doe", 123456)

        # User exists now
        assert self.dal.user_exists(123456) is True
        assert self.dal.user_exists(999999) is False

    def test_get_user_count(self):
        """Test getting the user count."""
        # Initially empty
        assert self.dal.get_user_count() == 0

        # Add users
        self.dal.create_user("John Doe", 123456)
        assert self.dal.get_user_count() == 1

        self.dal.create_user("Jane Smith", 789012)
        assert self.dal.get_user_count() == 2

        # Delete a user
        user_id = self.dal.create_user("Bob Wilson", 345678)
        assert self.dal.get_user_count() == 3

        self.dal.delete_user(user_id)
        assert self.dal.get_user_count() == 2

    def test_database_file_creation(self):
        """Test that database file is created properly."""
        # Create a new DAL with a specific path
        test_db_path = "test_database.db"

        try:
            dal = SpendingTrackerDAL(test_db_path)
            assert os.path.exists(test_db_path)

            # Test that the tables were created
            dal.create_user("Test User", 111111)
            user = dal.get_user_by_telegram_id(111111)
            assert user is not None
            assert user["name"] == "Test User"

        finally:
            # Clean up
            if os.path.exists(test_db_path):
                os.unlink(test_db_path)

    def test_transaction_rollback_on_error(self):
        """Test that transactions are properly handled."""
        # This test ensures that if an error occurs, the transaction is rolled back
        self.dal.create_user("John Doe", 123456)

        # Try to create a user with the same telegram_id
        initial_count = self.dal.get_user_count()

        with pytest.raises(sqlite3.IntegrityError):
            self.dal.create_user("Jane Doe", 123456)

        # Count should remain the same
        assert self.dal.get_user_count() == initial_count

    # =================
    # CURRENCY TESTS
    # =================

    def test_create_currency(self):
        """Test creating a new currency."""
        currency_id = self.dal.create_currency("USD")

        assert currency_id is not None
        assert currency_id > 0

        # Verify currency was created
        currency = self.dal.get_currency_by_id(currency_id)
        assert currency is not None
        assert currency["currency_code"] == "USD"

    def test_create_currency_case_insensitive(self):
        """Test creating currency with lowercase code converts to uppercase."""
        currency_id = self.dal.create_currency("eur")

        currency = self.dal.get_currency_by_id(currency_id)
        assert currency is not None
        assert currency["currency_code"] == "EUR"

    def test_create_currency_duplicate_code(self):
        """Test creating a currency with duplicate code raises error."""
        self.dal.create_currency("USD")

        with pytest.raises(sqlite3.IntegrityError):
            self.dal.create_currency("USD")

    def test_get_currency_by_id(self):
        """Test retrieving a currency by ID."""
        currency_id = self.dal.create_currency("EUR")

        currency = self.dal.get_currency_by_id(currency_id)
        assert currency is not None
        assert currency["id"] == currency_id
        assert currency["currency_code"] == "EUR"

    def test_get_currency_by_id_not_found(self):
        """Test retrieving a non-existent currency by ID."""
        currency = self.dal.get_currency_by_id(999)
        assert currency is None

    def test_get_currency_by_code(self):
        """Test retrieving a currency by code."""
        currency_id = self.dal.create_currency("BYN")

        currency = self.dal.get_currency_by_code("BYN")
        assert currency is not None
        assert currency["id"] == currency_id
        assert currency["currency_code"] == "BYN"

    def test_get_currency_by_code_case_insensitive(self):
        """Test retrieving currency by code is case insensitive."""
        currency_id = self.dal.create_currency("GBP")

        currency = self.dal.get_currency_by_code("gbp")
        assert currency is not None
        assert currency["id"] == currency_id
        assert currency["currency_code"] == "GBP"

    def test_get_currency_by_code_not_found(self):
        """Test retrieving a non-existent currency by code."""
        currency = self.dal.get_currency_by_code("XYZ")
        assert currency is None

    def test_get_all_currencies(self):
        """Test retrieving all currencies."""
        # Initially empty
        currencies = self.dal.get_all_currencies()
        assert len(currencies) == 0

        # Add some currencies
        self.dal.create_currency("USD")
        self.dal.create_currency("EUR")
        self.dal.create_currency("BYN")

        currencies = self.dal.get_all_currencies()
        assert len(currencies) == 3
        # Should be sorted by currency_code
        assert currencies[0]["currency_code"] == "BYN"
        assert currencies[1]["currency_code"] == "EUR"
        assert currencies[2]["currency_code"] == "USD"

    def test_update_currency(self):
        """Test updating a currency's code."""
        currency_id = self.dal.create_currency("OLD")

        # Update the currency
        result = self.dal.update_currency(currency_id, "NEW")
        assert result is True

        # Verify the update
        currency = self.dal.get_currency_by_id(currency_id)
        assert currency["currency_code"] == "NEW"

    def test_update_currency_not_found(self):
        """Test updating a non-existent currency."""
        result = self.dal.update_currency(999, "NEW")
        assert result is False

    def test_delete_currency(self):
        """Test deleting a currency."""
        currency_id = self.dal.create_currency("DELETE")

        # Delete the currency
        result = self.dal.delete_currency(currency_id)
        assert result is True

        # Verify the currency is gone
        currency = self.dal.get_currency_by_id(currency_id)
        assert currency is None

    def test_delete_currency_not_found(self):
        """Test deleting a non-existent currency."""
        result = self.dal.delete_currency(999)
        assert result is False

    def test_currency_exists(self):
        """Test checking if a currency exists."""
        # Currency doesn't exist yet
        assert self.dal.currency_exists("JPY") is False

        # Create currency
        self.dal.create_currency("JPY")

        # Currency exists now
        assert self.dal.currency_exists("JPY") is True
        assert self.dal.currency_exists("jpy") is True  # Case insensitive
        assert self.dal.currency_exists("XYZ") is False

    def test_get_currency_count(self):
        """Test getting the currency count."""
        # Initially empty
        assert self.dal.get_currency_count() == 0

        # Add currencies
        self.dal.create_currency("USD")
        assert self.dal.get_currency_count() == 1

        self.dal.create_currency("EUR")
        assert self.dal.get_currency_count() == 2

        # Delete a currency
        currency_id = self.dal.create_currency("BYN")
        assert self.dal.get_currency_count() == 3

        self.dal.delete_currency(currency_id)
        assert self.dal.get_currency_count() == 2

    # =================
    # ACCOUNT TESTS
    # =================

    def test_create_account(self):
        """Test creating a new account."""
        # First create a currency
        currency_id = self.dal.create_currency("USD")

        # Create account
        account_id = self.dal.create_account(currency_id, "My Main Account", "US1234567890")

        assert account_id is not None
        assert account_id > 0

        # Verify account was created
        account = self.dal.get_account_by_id(account_id)
        assert account is not None
        assert account["currency_id"] == currency_id
        assert account["name"] == "My Main Account"
        assert account["iban"] == "US1234567890"

    def test_create_account_without_iban(self):
        """Test creating an account without IBAN."""
        currency_id = self.dal.create_currency("EUR")

        account_id = self.dal.create_account(currency_id, "Cash Account")

        account = self.dal.get_account_by_id(account_id)
        assert account is not None
        assert account["currency_id"] == currency_id
        assert account["name"] == "Cash Account"
        assert account["iban"] is None

    def test_create_account_invalid_currency(self):
        """Test creating an account with invalid currency_id raises error."""
        with pytest.raises(sqlite3.IntegrityError):
            self.dal.create_account(999, "Invalid Account")

    def test_get_account_by_id(self):
        """Test retrieving an account by ID."""
        currency_id = self.dal.create_currency("BYN")
        account_id = self.dal.create_account(currency_id, "Test Account", "BY1234567890")

        account = self.dal.get_account_by_id(account_id)
        assert account is not None
        assert account["id"] == account_id
        assert account["currency_id"] == currency_id
        assert account["name"] == "Test Account"
        assert account["iban"] == "BY1234567890"

    def test_get_account_by_id_not_found(self):
        """Test retrieving a non-existent account by ID."""
        account = self.dal.get_account_by_id(999)
        assert account is None

    def test_get_account_by_id_with_currency(self):
        """Test retrieving an account with currency information."""
        currency_id = self.dal.create_currency("GBP")
        account_id = self.dal.create_account(currency_id, "UK Account", "GB1234567890")

        account = self.dal.get_account_by_id_with_currency(account_id)
        assert account is not None
        assert account["id"] == account_id
        assert account["currency_id"] == currency_id
        assert account["name"] == "UK Account"
        assert account["iban"] == "GB1234567890"
        assert account["currency_code"] == "GBP"

    def test_get_account_by_id_with_currency_not_found(self):
        """Test retrieving a non-existent account with currency info."""
        account = self.dal.get_account_by_id_with_currency(999)
        assert account is None

    def test_get_all_accounts(self):
        """Test retrieving all accounts."""
        # Initially empty
        accounts = self.dal.get_all_accounts()
        assert len(accounts) == 0

        # Create currency and accounts
        usd_id = self.dal.create_currency("USD")
        eur_id = self.dal.create_currency("EUR")

        self.dal.create_account(usd_id, "USD Account")
        self.dal.create_account(eur_id, "EUR Account")

        accounts = self.dal.get_all_accounts()
        assert len(accounts) == 2
        # Should be sorted by name
        assert accounts[0]["name"] == "EUR Account"
        assert accounts[1]["name"] == "USD Account"

    def test_get_all_accounts_with_currency(self):
        """Test retrieving all accounts with currency information."""
        # Create currency and accounts
        usd_id = self.dal.create_currency("USD")
        eur_id = self.dal.create_currency("EUR")

        self.dal.create_account(usd_id, "USD Account")
        self.dal.create_account(eur_id, "EUR Account")

        accounts = self.dal.get_all_accounts_with_currency()
        assert len(accounts) == 2
        # Should be sorted by name
        assert accounts[0]["name"] == "EUR Account"
        assert accounts[0]["currency_code"] == "EUR"
        assert accounts[1]["name"] == "USD Account"
        assert accounts[1]["currency_code"] == "USD"

    def test_get_accounts_by_currency(self):
        """Test retrieving accounts by currency."""
        usd_id = self.dal.create_currency("USD")
        eur_id = self.dal.create_currency("EUR")

        self.dal.create_account(usd_id, "USD Account 1")
        self.dal.create_account(usd_id, "USD Account 2")
        self.dal.create_account(eur_id, "EUR Account")

        usd_accounts = self.dal.get_accounts_by_currency(usd_id)
        assert len(usd_accounts) == 2
        assert usd_accounts[0]["name"] == "USD Account 1"
        assert usd_accounts[1]["name"] == "USD Account 2"

        eur_accounts = self.dal.get_accounts_by_currency(eur_id)
        assert len(eur_accounts) == 1
        assert eur_accounts[0]["name"] == "EUR Account"

    def test_update_account(self):
        """Test updating an account."""
        usd_id = self.dal.create_currency("USD")
        eur_id = self.dal.create_currency("EUR")

        account_id = self.dal.create_account(usd_id, "Original Account", "US1234567890")

        # Update the account
        result = self.dal.update_account(account_id, eur_id, "Updated Account", "EU0987654321")
        assert result is True

        # Verify the update
        account = self.dal.get_account_by_id(account_id)
        assert account["currency_id"] == eur_id
        assert account["name"] == "Updated Account"
        assert account["iban"] == "EU0987654321"

    def test_update_account_remove_iban(self):
        """Test updating an account to remove IBAN."""
        currency_id = self.dal.create_currency("USD")
        account_id = self.dal.create_account(currency_id, "Test Account", "US1234567890")

        # Update to remove IBAN
        result = self.dal.update_account(account_id, currency_id, "Test Account", None)
        assert result is True

        # Verify IBAN is removed
        account = self.dal.get_account_by_id(account_id)
        assert account["iban"] is None

    def test_update_account_not_found(self):
        """Test updating a non-existent account."""
        currency_id = self.dal.create_currency("USD")
        result = self.dal.update_account(999, currency_id, "New Name")
        assert result is False

    def test_delete_account(self):
        """Test deleting an account."""
        currency_id = self.dal.create_currency("USD")
        account_id = self.dal.create_account(currency_id, "Delete Me")

        # Delete the account
        result = self.dal.delete_account(account_id)
        assert result is True

        # Verify the account is gone
        account = self.dal.get_account_by_id(account_id)
        assert account is None

    def test_delete_account_not_found(self):
        """Test deleting a non-existent account."""
        result = self.dal.delete_account(999)
        assert result is False

    def test_account_exists(self):
        """Test checking if an account exists."""
        currency_id = self.dal.create_currency("USD")

        # Account doesn't exist yet
        assert self.dal.account_exists(999) is False

        # Create account
        account_id = self.dal.create_account(currency_id, "Test Account")

        # Account exists now
        assert self.dal.account_exists(account_id) is True
        assert self.dal.account_exists(999) is False

    def test_get_account_count(self):
        """Test getting the account count."""
        # Initially empty
        assert self.dal.get_account_count() == 0

        # Add accounts
        currency_id = self.dal.create_currency("USD")
        self.dal.create_account(currency_id, "Account 1")
        assert self.dal.get_account_count() == 1

        self.dal.create_account(currency_id, "Account 2")
        assert self.dal.get_account_count() == 2

        # Delete an account
        account_id = self.dal.create_account(currency_id, "Account 3")
        assert self.dal.get_account_count() == 3

        self.dal.delete_account(account_id)
        assert self.dal.get_account_count() == 2

    # =================
    # CATEGORY TESTS
    # =================

    def test_create_category(self):
        """Test creating a new category."""
        category_id = self.dal.create_category("Food")

        assert category_id is not None
        assert category_id > 0

        # Verify category was created
        category = self.dal.get_category_by_id(category_id)
        assert category is not None
        assert category["name"] == "Food"
        assert category["sort_order"] == 0

    def test_create_category_with_sort_order(self):
        """Test creating a category with custom sort order."""
        category_id = self.dal.create_category("Transport", 10)

        category = self.dal.get_category_by_id(category_id)
        assert category is not None
        assert category["name"] == "Transport"
        assert category["sort_order"] == 10

    def test_create_category_duplicate_name(self):
        """Test creating a category with duplicate name raises error."""
        self.dal.create_category("Food")

        with pytest.raises(sqlite3.IntegrityError):
            self.dal.create_category("Food")

    def test_get_category_by_id(self):
        """Test retrieving a category by ID."""
        category_id = self.dal.create_category("Entertainment", 5)

        category = self.dal.get_category_by_id(category_id)
        assert category is not None
        assert category["id"] == category_id
        assert category["name"] == "Entertainment"
        assert category["sort_order"] == 5

    def test_get_category_by_id_not_found(self):
        """Test retrieving a non-existent category by ID."""
        category = self.dal.get_category_by_id(999)
        assert category is None

    def test_get_category_by_name(self):
        """Test retrieving a category by name."""
        category_id = self.dal.create_category("Healthcare", 3)

        category = self.dal.get_category_by_name("Healthcare")
        assert category is not None
        assert category["id"] == category_id
        assert category["name"] == "Healthcare"
        assert category["sort_order"] == 3

    def test_get_category_by_name_not_found(self):
        """Test retrieving a non-existent category by name."""
        category = self.dal.get_category_by_name("NonExistent")
        assert category is None

    def test_get_all_categories(self):
        """Test retrieving all categories."""
        # Initially empty
        categories = self.dal.get_all_categories()
        assert len(categories) == 0

        # Add categories with different sort orders
        self.dal.create_category("Food", 1)
        self.dal.create_category("Transport", 2)
        self.dal.create_category("Entertainment", 1)  # Same sort order as Food

        categories = self.dal.get_all_categories()
        assert len(categories) == 3
        # Should be sorted by sort_order, then name
        assert categories[0]["name"] == "Entertainment"  # sort_order=1, name comes first
        assert categories[1]["name"] == "Food"  # sort_order=1, name comes second
        assert categories[2]["name"] == "Transport"  # sort_order=2

    def test_update_category(self):
        """Test updating a category."""
        category_id = self.dal.create_category("Food", 1)

        # Update the category
        result = self.dal.update_category(category_id, "Groceries", 5)
        assert result is True

        # Verify the update
        category = self.dal.get_category_by_id(category_id)
        assert category["name"] == "Groceries"
        assert category["sort_order"] == 5

    def test_update_category_not_found(self):
        """Test updating a non-existent category."""
        result = self.dal.update_category(999, "New Name", 1)
        assert result is False

    def test_delete_category(self):
        """Test deleting a category."""
        category_id = self.dal.create_category("Delete Me", 1)

        # Delete the category
        result = self.dal.delete_category(category_id)
        assert result is True

        # Verify the category is gone
        category = self.dal.get_category_by_id(category_id)
        assert category is None

    def test_delete_category_not_found(self):
        """Test deleting a non-existent category."""
        result = self.dal.delete_category(999)
        assert result is False

    def test_category_exists(self):
        """Test checking if a category exists."""
        # Category doesn't exist yet
        assert self.dal.category_exists("Food") is False

        # Create category
        self.dal.create_category("Food")

        # Category exists now
        assert self.dal.category_exists("Food") is True
        assert self.dal.category_exists("Transport") is False

    def test_get_category_count(self):
        """Test getting the category count."""
        # Initially empty
        assert self.dal.get_category_count() == 0

        # Add categories
        self.dal.create_category("Food")
        assert self.dal.get_category_count() == 1

        self.dal.create_category("Transport")
        assert self.dal.get_category_count() == 2

        # Delete a category
        category_id = self.dal.create_category("Entertainment")
        assert self.dal.get_category_count() == 3

        self.dal.delete_category(category_id)
        assert self.dal.get_category_count() == 2

    def test_reorder_categories(self):
        """Test reordering categories."""
        # Create categories
        food_id = self.dal.create_category("Food", 1)
        transport_id = self.dal.create_category("Transport", 2)
        entertainment_id = self.dal.create_category("Entertainment", 3)

        # Reorder categories
        result = self.dal.reorder_categories([(food_id, 3), (transport_id, 1), (entertainment_id, 2)])
        assert result is True

        # Verify new order
        categories = self.dal.get_all_categories()
        assert len(categories) == 3
        assert categories[0]["name"] == "Transport"  # sort_order=1
        assert categories[1]["name"] == "Entertainment"  # sort_order=2
        assert categories[2]["name"] == "Food"  # sort_order=3

    def test_reorder_categories_invalid_id(self):
        """Test reordering categories with invalid ID."""
        food_id = self.dal.create_category("Food", 1)

        # Try to reorder with invalid category ID
        result = self.dal.reorder_categories([(food_id, 2), (999, 1)])  # Invalid ID
        assert result is False

        # Verify original order is maintained
        category = self.dal.get_category_by_id(food_id)
        assert category["sort_order"] == 1

    def test_reorder_categories_empty_list(self):
        """Test reordering categories with empty list."""
        result = self.dal.reorder_categories([])
        assert result is True

    # =================
    # SPENDING TESTS
    # =================

    def _create_spending_dependencies(self):
        """Helper method to create spending dependencies."""
        currency_id = self.dal.create_currency("USD")
        account_id = self.dal.create_account(currency_id, "Test Account")
        category_id = self.dal.create_category("Food")
        user_id = self.dal.create_user("Test User", 12345)
        return currency_id, account_id, category_id, user_id

    def test_create_spending(self):
        """Test creating a new spending entry."""
        currency_id, account_id, category_id, user_id = self._create_spending_dependencies()

        spending_id = self.dal.create_spending(
            account_id=account_id,
            amount=25.50,
            category_id=category_id,
            reporter_id=user_id,
            notes="Lunch at restaurant",
        )

        assert spending_id is not None
        assert spending_id > 0

        # Verify spending was created
        spending = self.dal.get_spending_by_id(spending_id)
        assert spending is not None
        assert spending["amount"] == 25.50
        assert spending["account_id"] == account_id
        assert spending["category_id"] == category_id
        assert spending["reporter_id"] == user_id
        assert spending["notes"] == "Lunch at restaurant"
        assert spending["timestamp"] is not None

    def test_create_spending_with_timestamp(self):
        """Test creating a spending entry with custom timestamp."""
        currency_id, account_id, category_id, user_id = self._create_spending_dependencies()
        custom_timestamp = datetime(2023, 12, 25, 15, 30, 0)

        spending_id = self.dal.create_spending(
            account_id=account_id,
            amount=100.00,
            category_id=category_id,
            reporter_id=user_id,
            notes="Christmas gift",
            timestamp=custom_timestamp,
        )

        spending = self.dal.get_spending_by_id(spending_id)
        assert spending is not None
        # SQLite stores datetime without 'T' separator
        assert spending["timestamp"] == custom_timestamp.strftime("%Y-%m-%d %H:%M:%S")

    def test_create_spending_without_notes(self):
        """Test creating a spending entry without notes."""
        currency_id, account_id, category_id, user_id = self._create_spending_dependencies()

        spending_id = self.dal.create_spending(
            account_id=account_id,
            amount=15.00,
            category_id=category_id,
            reporter_id=user_id,
        )

        spending = self.dal.get_spending_by_id(spending_id)
        assert spending is not None
        assert spending["notes"] is None

    def test_create_spending_invalid_account(self):
        """Test creating a spending entry with invalid account ID raises error."""
        currency_id, account_id, category_id, user_id = self._create_spending_dependencies()

        with pytest.raises(sqlite3.IntegrityError):
            self.dal.create_spending(
                account_id=999,  # Invalid account ID
                amount=25.50,
                category_id=category_id,
                reporter_id=user_id,
            )

    def test_get_spending_by_id(self):
        """Test retrieving a spending entry by ID."""
        currency_id, account_id, category_id, user_id = self._create_spending_dependencies()

        spending_id = self.dal.create_spending(
            account_id=account_id,
            amount=50.75,
            category_id=category_id,
            reporter_id=user_id,
            notes="Grocery shopping",
        )

        spending = self.dal.get_spending_by_id(spending_id)
        assert spending is not None
        assert spending["id"] == spending_id
        assert spending["amount"] == 50.75
        assert spending["notes"] == "Grocery shopping"

    def test_get_spending_by_id_not_found(self):
        """Test retrieving a non-existent spending entry by ID."""
        spending = self.dal.get_spending_by_id(999)
        assert spending is None

    def test_get_spending_by_id_with_details(self):
        """Test retrieving a spending entry with details."""
        currency_id, account_id, category_id, user_id = self._create_spending_dependencies()

        spending_id = self.dal.create_spending(
            account_id=account_id,
            amount=75.25,
            category_id=category_id,
            reporter_id=user_id,
            notes="Dinner out",
        )

        spending = self.dal.get_spending_by_id_with_details(spending_id)
        assert spending is not None
        assert spending["id"] == spending_id
        assert spending["amount"] == 75.25
        assert spending["notes"] == "Dinner out"
        assert spending["account_name"] == "Test Account"
        assert spending["category_name"] == "Food"
        assert spending["reporter_name"] == "Test User"
        assert spending["reporter_telegram_id"] == 12345
        assert spending["currency_code"] == "USD"

    def test_get_spending_by_id_with_details_not_found(self):
        """Test retrieving a non-existent spending entry with details."""
        spending = self.dal.get_spending_by_id_with_details(999)
        assert spending is None

    def test_get_all_spending(self):
        """Test retrieving all spending entries."""
        currency_id, account_id, category_id, user_id = self._create_spending_dependencies()

        # Initially empty
        spending_list = self.dal.get_all_spending()
        assert len(spending_list) == 0

        # Create spending entries
        spending1_id = self.dal.create_spending(account_id, 10.00, category_id, user_id, "Coffee")
        spending2_id = self.dal.create_spending(account_id, 20.00, category_id, user_id, "Lunch")

        spending_list = self.dal.get_all_spending()
        assert len(spending_list) == 2
        # Should be sorted by timestamp descending (newest first)
        assert spending_list[0]["id"] == spending2_id
        assert spending_list[1]["id"] == spending1_id

    def test_get_all_spending_with_limit(self):
        """Test retrieving all spending entries with limit."""
        currency_id, account_id, category_id, user_id = self._create_spending_dependencies()

        # Create multiple spending entries
        for i in range(5):
            self.dal.create_spending(account_id, 10.00 + i, category_id, user_id, f"Purchase {i}")

        spending_list = self.dal.get_all_spending(limit=3)
        assert len(spending_list) == 3

    def test_get_all_spending_with_details(self):
        """Test retrieving all spending entries with details."""
        currency_id, account_id, category_id, user_id = self._create_spending_dependencies()

        spending_id = self.dal.create_spending(account_id, 30.00, category_id, user_id, "Groceries")

        spending_list = self.dal.get_all_spending_with_details()
        assert len(spending_list) == 1
        assert spending_list[0]["id"] == spending_id
        assert spending_list[0]["account_name"] == "Test Account"
        assert spending_list[0]["category_name"] == "Food"
        assert spending_list[0]["reporter_name"] == "Test User"
        assert spending_list[0]["currency_code"] == "USD"

    def test_get_spending_by_account(self):
        """Test retrieving spending entries by account."""
        currency_id, account_id, category_id, user_id = self._create_spending_dependencies()

        # Create another account
        account2_id = self.dal.create_account(currency_id, "Account 2")

        # Create spending for both accounts
        spending1_id = self.dal.create_spending(account_id, 10.00, category_id, user_id, "Account 1")
        spending2_id = self.dal.create_spending(account2_id, 20.00, category_id, user_id, "Account 2")

        # Get spending for account 1
        spending_list = self.dal.get_spending_by_account(account_id)
        assert len(spending_list) == 1
        assert spending_list[0]["id"] == spending1_id

        # Get spending for account 2
        spending_list = self.dal.get_spending_by_account(account2_id)
        assert len(spending_list) == 1
        assert spending_list[0]["id"] == spending2_id

    def test_get_spending_by_category(self):
        """Test retrieving spending entries by category."""
        currency_id, account_id, category_id, user_id = self._create_spending_dependencies()

        # Create another category
        category2_id = self.dal.create_category("Transport")

        # Create spending for both categories
        spending1_id = self.dal.create_spending(account_id, 10.00, category_id, user_id, "Food")
        spending2_id = self.dal.create_spending(account_id, 20.00, category2_id, user_id, "Transport")

        # Get spending for Food category
        spending_list = self.dal.get_spending_by_category(category_id)
        assert len(spending_list) == 1
        assert spending_list[0]["id"] == spending1_id

        # Get spending for Transport category
        spending_list = self.dal.get_spending_by_category(category2_id)
        assert len(spending_list) == 1
        assert spending_list[0]["id"] == spending2_id

    def test_get_spending_by_reporter(self):
        """Test retrieving spending entries by reporter."""
        currency_id, account_id, category_id, user_id = self._create_spending_dependencies()

        # Create another user
        user2_id = self.dal.create_user("User 2", 67890)

        # Create spending for both users
        spending1_id = self.dal.create_spending(account_id, 10.00, category_id, user_id, "User 1")
        spending2_id = self.dal.create_spending(account_id, 20.00, category_id, user2_id, "User 2")

        # Get spending for user 1
        spending_list = self.dal.get_spending_by_reporter(user_id)
        assert len(spending_list) == 1
        assert spending_list[0]["id"] == spending1_id

        # Get spending for user 2
        spending_list = self.dal.get_spending_by_reporter(user2_id)
        assert len(spending_list) == 1
        assert spending_list[0]["id"] == spending2_id

    def test_get_spending_by_date_range(self):
        """Test retrieving spending entries by date range."""
        currency_id, account_id, category_id, user_id = self._create_spending_dependencies()

        # Create spending entries with different timestamps
        now = datetime.now()
        yesterday = now - timedelta(days=1)
        tomorrow = now + timedelta(days=1)

        self.dal.create_spending(account_id, 10.00, category_id, user_id, "Yesterday", yesterday)
        spending2_id = self.dal.create_spending(account_id, 20.00, category_id, user_id, "Today", now)
        self.dal.create_spending(account_id, 30.00, category_id, user_id, "Tomorrow", tomorrow)

        # Get spending for today only
        spending_list = self.dal.get_spending_by_date_range(now, now + timedelta(hours=1))
        assert len(spending_list) == 1
        assert spending_list[0]["id"] == spending2_id

        # Get spending for yesterday and today
        spending_list = self.dal.get_spending_by_date_range(yesterday, now + timedelta(hours=1))
        assert len(spending_list) == 2

    def test_update_spending(self):
        """Test updating a spending entry."""
        currency_id, account_id, category_id, user_id = self._create_spending_dependencies()

        # Create another category
        category2_id = self.dal.create_category("Transport")

        spending_id = self.dal.create_spending(account_id, 10.00, category_id, user_id, "Original")

        # Update the spending
        result = self.dal.update_spending(spending_id, account_id, 25.00, category2_id, user_id, "Updated")
        assert result is True

        # Verify the update
        spending = self.dal.get_spending_by_id(spending_id)
        assert spending["amount"] == 25.00
        assert spending["category_id"] == category2_id
        assert spending["notes"] == "Updated"

    def test_update_spending_not_found(self):
        """Test updating a non-existent spending entry."""
        currency_id, account_id, category_id, user_id = self._create_spending_dependencies()

        result = self.dal.update_spending(999, account_id, 25.00, category_id, user_id, "Updated")
        assert result is False

    def test_delete_spending(self):
        """Test deleting a spending entry."""
        currency_id, account_id, category_id, user_id = self._create_spending_dependencies()

        spending_id = self.dal.create_spending(account_id, 10.00, category_id, user_id, "Delete me")

        # Delete the spending
        result = self.dal.delete_spending(spending_id)
        assert result is True

        # Verify the spending is gone
        spending = self.dal.get_spending_by_id(spending_id)
        assert spending is None

    def test_delete_spending_not_found(self):
        """Test deleting a non-existent spending entry."""
        result = self.dal.delete_spending(999)
        assert result is False

    def test_spending_exists(self):
        """Test checking if a spending entry exists."""
        currency_id, account_id, category_id, user_id = self._create_spending_dependencies()

        # Spending doesn't exist yet
        assert self.dal.spending_exists(999) is False

        # Create spending
        spending_id = self.dal.create_spending(account_id, 10.00, category_id, user_id, "Test")

        # Spending exists now
        assert self.dal.spending_exists(spending_id) is True
        assert self.dal.spending_exists(999) is False

    def test_get_spending_count(self):
        """Test getting the spending count."""
        currency_id, account_id, category_id, user_id = self._create_spending_dependencies()

        # Initially empty
        assert self.dal.get_spending_count() == 0

        # Add spending entries
        self.dal.create_spending(account_id, 10.00, category_id, user_id, "First")
        assert self.dal.get_spending_count() == 1

        self.dal.create_spending(account_id, 20.00, category_id, user_id, "Second")
        assert self.dal.get_spending_count() == 2

        # Delete a spending entry
        spending_id = self.dal.create_spending(account_id, 30.00, category_id, user_id, "Third")
        assert self.dal.get_spending_count() == 3

        self.dal.delete_spending(spending_id)
        assert self.dal.get_spending_count() == 2

    def test_get_spending_total_by_account(self):
        """Test getting total spending by account."""
        currency_id, account_id, category_id, user_id = self._create_spending_dependencies()

        # Create another account
        account2_id = self.dal.create_account(currency_id, "Account 2")

        # Initially zero
        assert self.dal.get_spending_total_by_account(account_id) == 0.0

        # Add spending to account 1
        self.dal.create_spending(account_id, 10.00, category_id, user_id, "First")
        self.dal.create_spending(account_id, 20.00, category_id, user_id, "Second")

        # Add spending to account 2
        self.dal.create_spending(account2_id, 30.00, category_id, user_id, "Third")

        # Check totals
        assert self.dal.get_spending_total_by_account(account_id) == 30.0
        assert self.dal.get_spending_total_by_account(account2_id) == 30.0

    def test_get_spending_total_by_category(self):
        """Test getting total spending by category."""
        currency_id, account_id, category_id, user_id = self._create_spending_dependencies()

        # Create another category
        category2_id = self.dal.create_category("Transport")

        # Initially zero
        assert self.dal.get_spending_total_by_category(category_id) == 0.0

        # Add spending to Food category
        self.dal.create_spending(account_id, 15.00, category_id, user_id, "Food 1")
        self.dal.create_spending(account_id, 25.00, category_id, user_id, "Food 2")

        # Add spending to Transport category
        self.dal.create_spending(account_id, 35.00, category2_id, user_id, "Transport 1")

        # Check totals
        assert self.dal.get_spending_total_by_category(category_id) == 40.0
        assert self.dal.get_spending_total_by_category(category2_id) == 35.0

    def test_get_spending_total_by_reporter(self):
        """Test getting total spending by reporter."""
        currency_id, account_id, category_id, user_id = self._create_spending_dependencies()

        # Create another user
        user2_id = self.dal.create_user("User 2", 67890)

        # Initially zero
        assert self.dal.get_spending_total_by_reporter(user_id) == 0.0

        # Add spending by user 1
        self.dal.create_spending(account_id, 12.00, category_id, user_id, "User 1 - First")
        self.dal.create_spending(account_id, 18.00, category_id, user_id, "User 1 - Second")

        # Add spending by user 2
        self.dal.create_spending(account_id, 25.00, category_id, user2_id, "User 2 - First")

        # Check totals
        assert self.dal.get_spending_total_by_reporter(user_id) == 30.0
        assert self.dal.get_spending_total_by_reporter(user2_id) == 25.0

    def test_get_spending_total_by_date_range(self):
        """Test getting total spending by date range."""
        currency_id, account_id, category_id, user_id = self._create_spending_dependencies()

        # Create spending entries with different timestamps
        now = datetime.now()
        yesterday = now - timedelta(days=1)
        tomorrow = now + timedelta(days=1)

        self.dal.create_spending(account_id, 10.00, category_id, user_id, "Yesterday", yesterday)
        self.dal.create_spending(account_id, 20.00, category_id, user_id, "Today", now)
        self.dal.create_spending(account_id, 30.00, category_id, user_id, "Tomorrow", tomorrow)

        # Get total for today only
        total = self.dal.get_spending_total_by_date_range(now, now + timedelta(hours=1))
        assert total == 20.0

        # Get total for yesterday and today
        total = self.dal.get_spending_total_by_date_range(yesterday, now + timedelta(hours=1))
        assert total == 30.0

        # Get total for all dates
        total = self.dal.get_spending_total_by_date_range(yesterday, tomorrow + timedelta(hours=1))
        assert total == 60.0
