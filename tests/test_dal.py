"""Tests for the Data Access Layer."""

import os
import sqlite3
import tempfile

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
        account_id = self.dal.create_account(
            currency_id, "My Main Account", "US1234567890"
        )

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
        account_id = self.dal.create_account(
            currency_id, "Test Account", "BY1234567890"
        )

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
        result = self.dal.update_account(
            account_id, eur_id, "Updated Account", "EU0987654321"
        )
        assert result is True

        # Verify the update
        account = self.dal.get_account_by_id(account_id)
        assert account["currency_id"] == eur_id
        assert account["name"] == "Updated Account"
        assert account["iban"] == "EU0987654321"

    def test_update_account_remove_iban(self):
        """Test updating an account to remove IBAN."""
        currency_id = self.dal.create_currency("USD")
        account_id = self.dal.create_account(
            currency_id, "Test Account", "US1234567890"
        )

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
