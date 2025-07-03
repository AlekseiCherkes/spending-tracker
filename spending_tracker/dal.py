"""
Data Access Layer for Spending Tracker
Uses sqlite3 directly for simple, transparent database operations.
"""

import sqlite3
import os
from typing import Optional, List, Tuple


class UserDAL:
    """Data Access Layer for Users table."""
    
    def __init__(self, db_path: str = "spending_tracker.db"):
        """Initialize the DAL with database path."""
        self.db_path = db_path
        self._create_tables()
    
    def _get_connection(self) -> sqlite3.Connection:
        """Get a database connection."""
        conn = sqlite3.connect(self.db_path)
        conn.row_factory = sqlite3.Row  # Enable dict-like access to rows
        return conn
    
    def _create_tables(self) -> None:
        """Create the users table if it doesn't exist."""
        with self._get_connection() as conn:
            conn.execute("""
                CREATE TABLE IF NOT EXISTS users (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL,
                    telegram_id INTEGER NOT NULL UNIQUE
                )
            """)
            conn.commit()
    
    def create_user(self, name: str, telegram_id: int) -> int:
        """
        Create a new user.
        
        Args:
            name: User's name
            telegram_id: Telegram user ID
            
        Returns:
            The ID of the created user
            
        Raises:
            sqlite3.IntegrityError: If telegram_id already exists
        """
        with self._get_connection() as conn:
            cursor = conn.execute(
                "INSERT INTO users (name, telegram_id) VALUES (?, ?)",
                (name, telegram_id)
            )
            conn.commit()
            return cursor.lastrowid
    
    def get_user_by_id(self, user_id: int) -> Optional[dict]:
        """
        Get a user by their ID.
        
        Args:
            user_id: User ID
            
        Returns:
            User data as dict or None if not found
        """
        with self._get_connection() as conn:
            cursor = conn.execute(
                "SELECT id, name, telegram_id FROM users WHERE id = ?",
                (user_id,)
            )
            row = cursor.fetchone()
            return dict(row) if row else None
    
    def get_user_by_telegram_id(self, telegram_id: int) -> Optional[dict]:
        """
        Get a user by their Telegram ID.
        
        Args:
            telegram_id: Telegram user ID
            
        Returns:
            User data as dict or None if not found
        """
        with self._get_connection() as conn:
            cursor = conn.execute(
                "SELECT id, name, telegram_id FROM users WHERE telegram_id = ?",
                (telegram_id,)
            )
            row = cursor.fetchone()
            return dict(row) if row else None
    
    def get_all_users(self) -> List[dict]:
        """
        Get all users.
        
        Returns:
            List of user data as dicts
        """
        with self._get_connection() as conn:
            cursor = conn.execute(
                "SELECT id, name, telegram_id FROM users ORDER BY id"
            )
            rows = cursor.fetchall()
            return [dict(row) for row in rows]
    
    def update_user(self, user_id: int, name: str) -> bool:
        """
        Update a user's name.
        
        Args:
            user_id: User ID
            name: New name
            
        Returns:
            True if user was updated, False if not found
        """
        with self._get_connection() as conn:
            cursor = conn.execute(
                "UPDATE users SET name = ? WHERE id = ?",
                (name, user_id)
            )
            conn.commit()
            return cursor.rowcount > 0
    
    def delete_user(self, user_id: int) -> bool:
        """
        Delete a user.
        
        Args:
            user_id: User ID
            
        Returns:
            True if user was deleted, False if not found
        """
        with self._get_connection() as conn:
            cursor = conn.execute(
                "DELETE FROM users WHERE id = ?",
                (user_id,)
            )
            conn.commit()
            return cursor.rowcount > 0
    
    def user_exists(self, telegram_id: int) -> bool:
        """
        Check if a user exists by Telegram ID.
        
        Args:
            telegram_id: Telegram user ID
            
        Returns:
            True if user exists, False otherwise
        """
        with self._get_connection() as conn:
            cursor = conn.execute(
                "SELECT 1 FROM users WHERE telegram_id = ?",
                (telegram_id,)
            )
            return cursor.fetchone() is not None
    
    def get_user_count(self) -> int:
        """
        Get the total number of users.
        
        Returns:
            Number of users
        """
        with self._get_connection() as conn:
            cursor = conn.execute("SELECT COUNT(*) FROM users")
            return cursor.fetchone()[0] 