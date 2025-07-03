"""Tests for the Data Access Layer."""

import pytest
import sqlite3
import os
import tempfile
from spending_tracker.dal import UserDAL


class TestUserDAL:
    """Test the UserDAL class."""
    
    def setup_method(self):
        """Set up test database for each test."""
        # Create a temporary database file for testing
        self.db_fd, self.db_path = tempfile.mkstemp(suffix='.db')
        os.close(self.db_fd)  # Close the file descriptor
        self.user_dal = UserDAL(self.db_path)
    
    def teardown_method(self):
        """Clean up after each test."""
        # Remove the temporary database file
        if os.path.exists(self.db_path):
            os.unlink(self.db_path)
    
    def test_create_user(self):
        """Test creating a new user."""
        user_id = self.user_dal.create_user("John Doe", 123456)
        
        assert user_id is not None
        assert user_id > 0
        
        # Verify user was created
        user = self.user_dal.get_user_by_id(user_id)
        assert user is not None
        assert user['name'] == "John Doe"
        assert user['telegram_id'] == 123456
    
    def test_create_user_duplicate_telegram_id(self):
        """Test creating a user with duplicate telegram_id raises error."""
        self.user_dal.create_user("John Doe", 123456)
        
        with pytest.raises(sqlite3.IntegrityError):
            self.user_dal.create_user("Jane Doe", 123456)
    
    def test_get_user_by_id(self):
        """Test retrieving a user by ID."""
        user_id = self.user_dal.create_user("John Doe", 123456)
        
        user = self.user_dal.get_user_by_id(user_id)
        assert user is not None
        assert user['id'] == user_id
        assert user['name'] == "John Doe"
        assert user['telegram_id'] == 123456
    
    def test_get_user_by_id_not_found(self):
        """Test retrieving a non-existent user by ID."""
        user = self.user_dal.get_user_by_id(999)
        assert user is None
    
    def test_get_user_by_telegram_id(self):
        """Test retrieving a user by Telegram ID."""
        user_id = self.user_dal.create_user("John Doe", 123456)
        
        user = self.user_dal.get_user_by_telegram_id(123456)
        assert user is not None
        assert user['id'] == user_id
        assert user['name'] == "John Doe"
        assert user['telegram_id'] == 123456
    
    def test_get_user_by_telegram_id_not_found(self):
        """Test retrieving a non-existent user by Telegram ID."""
        user = self.user_dal.get_user_by_telegram_id(999999)
        assert user is None
    
    def test_get_all_users(self):
        """Test retrieving all users."""
        # Initially empty
        users = self.user_dal.get_all_users()
        assert len(users) == 0
        
        # Add some users
        self.user_dal.create_user("John Doe", 123456)
        self.user_dal.create_user("Jane Smith", 789012)
        
        users = self.user_dal.get_all_users()
        assert len(users) == 2
        assert users[0]['name'] == "John Doe"
        assert users[1]['name'] == "Jane Smith"
    
    def test_update_user(self):
        """Test updating a user's name."""
        user_id = self.user_dal.create_user("John Doe", 123456)
        
        # Update the user
        result = self.user_dal.update_user(user_id, "John Smith")
        assert result is True
        
        # Verify the update
        user = self.user_dal.get_user_by_id(user_id)
        assert user['name'] == "John Smith"
        assert user['telegram_id'] == 123456  # Should not change
    
    def test_update_user_not_found(self):
        """Test updating a non-existent user."""
        result = self.user_dal.update_user(999, "New Name")
        assert result is False
    
    def test_delete_user(self):
        """Test deleting a user."""
        user_id = self.user_dal.create_user("John Doe", 123456)
        
        # Delete the user
        result = self.user_dal.delete_user(user_id)
        assert result is True
        
        # Verify the user is gone
        user = self.user_dal.get_user_by_id(user_id)
        assert user is None
    
    def test_delete_user_not_found(self):
        """Test deleting a non-existent user."""
        result = self.user_dal.delete_user(999)
        assert result is False
    
    def test_user_exists(self):
        """Test checking if a user exists."""
        # User doesn't exist yet
        assert self.user_dal.user_exists(123456) is False
        
        # Create user
        self.user_dal.create_user("John Doe", 123456)
        
        # User exists now
        assert self.user_dal.user_exists(123456) is True
        assert self.user_dal.user_exists(999999) is False
    
    def test_get_user_count(self):
        """Test getting the user count."""
        # Initially empty
        assert self.user_dal.get_user_count() == 0
        
        # Add users
        self.user_dal.create_user("John Doe", 123456)
        assert self.user_dal.get_user_count() == 1
        
        self.user_dal.create_user("Jane Smith", 789012)
        assert self.user_dal.get_user_count() == 2
        
        # Delete a user
        user_id = self.user_dal.create_user("Bob Wilson", 345678)
        assert self.user_dal.get_user_count() == 3
        
        self.user_dal.delete_user(user_id)
        assert self.user_dal.get_user_count() == 2
    
    def test_database_file_creation(self):
        """Test that database file is created properly."""
        # Create a new DAL with a specific path
        test_db_path = "test_database.db"
        
        try:
            dal = UserDAL(test_db_path)
            assert os.path.exists(test_db_path)
            
            # Test that the tables were created
            dal.create_user("Test User", 111111)
            user = dal.get_user_by_telegram_id(111111)
            assert user is not None
            assert user['name'] == "Test User"
            
        finally:
            # Clean up
            if os.path.exists(test_db_path):
                os.unlink(test_db_path)
    
    def test_transaction_rollback_on_error(self):
        """Test that transactions are properly handled."""
        # This test ensures that if an error occurs, the transaction is rolled back
        self.user_dal.create_user("John Doe", 123456)
        
        # Try to create a user with the same telegram_id
        initial_count = self.user_dal.get_user_count()
        
        with pytest.raises(sqlite3.IntegrityError):
            self.user_dal.create_user("Jane Doe", 123456)
        
        # Count should remain the same
        assert self.user_dal.get_user_count() == initial_count 