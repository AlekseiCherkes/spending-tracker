"""
Data Access Layer for Spending Tracker
Uses sqlite3 directly for simple, transparent database operations.
"""

import sqlite3
import os
from typing import Optional, List, Tuple


class SpendingTrackerDAL:
    """Unified Data Access Layer for all Spending Tracker tables."""
    
    def __init__(self, db_path: str = "spending_tracker.db"):
        """Initialize the DAL with database path."""
        self.db_path = db_path
        self._create_tables()
    
    def _get_connection(self) -> sqlite3.Connection:
        """Get a database connection."""
        conn = sqlite3.connect(self.db_path)
        conn.row_factory = sqlite3.Row  # Enable dict-like access to rows
        conn.execute("PRAGMA foreign_keys = ON")  # Enable foreign key constraints
        return conn
    
    def _create_tables(self) -> None:
        """Create all tables if they don't exist."""
        with self._get_connection() as conn:
            # Users table
            conn.execute("""
                CREATE TABLE IF NOT EXISTS users (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL,
                    telegram_id INTEGER NOT NULL UNIQUE
                )
            """)
            
            # Currencies table
            conn.execute("""
                CREATE TABLE IF NOT EXISTS currencies (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    currency_code TEXT NOT NULL UNIQUE
                )
            """)
            
            # Accounts table
            conn.execute("""
                CREATE TABLE IF NOT EXISTS accounts (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    currency_id INTEGER NOT NULL,
                    iban TEXT,
                    name TEXT NOT NULL,
                    FOREIGN KEY (currency_id) REFERENCES currencies (id)
                )
            """)
            
            conn.commit()
    
    # =================
    # CURRENCY OPERATIONS
    # =================
    
    def create_currency(self, currency_code: str) -> int:
        """
        Create a new currency.
        
        Args:
            currency_code: Currency code (e.g., EUR, USD, BYN)
            
        Returns:
            The ID of the created currency
            
        Raises:
            sqlite3.IntegrityError: If currency_code already exists
        """
        with self._get_connection() as conn:
            cursor = conn.execute(
                "INSERT INTO currencies (currency_code) VALUES (?)",
                (currency_code.upper(),)
            )
            conn.commit()
            return cursor.lastrowid
    
    def get_currency_by_id(self, currency_id: int) -> Optional[dict]:
        """
        Get a currency by its ID.
        
        Args:
            currency_id: Currency ID
            
        Returns:
            Currency data as dict or None if not found
        """
        with self._get_connection() as conn:
            cursor = conn.execute(
                "SELECT id, currency_code FROM currencies WHERE id = ?",
                (currency_id,)
            )
            row = cursor.fetchone()
            return dict(row) if row else None
    
    def get_currency_by_code(self, currency_code: str) -> Optional[dict]:
        """
        Get a currency by its code.
        
        Args:
            currency_code: Currency code (e.g., EUR, USD)
            
        Returns:
            Currency data as dict or None if not found
        """
        with self._get_connection() as conn:
            cursor = conn.execute(
                "SELECT id, currency_code FROM currencies WHERE currency_code = ?",
                (currency_code.upper(),)
            )
            row = cursor.fetchone()
            return dict(row) if row else None
    
    def get_all_currencies(self) -> List[dict]:
        """
        Get all currencies.
        
        Returns:
            List of currency data as dicts, sorted by currency_code
        """
        with self._get_connection() as conn:
            cursor = conn.execute(
                "SELECT id, currency_code FROM currencies ORDER BY currency_code"
            )
            rows = cursor.fetchall()
            return [dict(row) for row in rows]
    
    def update_currency(self, currency_id: int, currency_code: str) -> bool:
        """
        Update a currency's code.
        
        Args:
            currency_id: Currency ID
            currency_code: New currency code
            
        Returns:
            True if currency was updated, False if not found
        """
        with self._get_connection() as conn:
            cursor = conn.execute(
                "UPDATE currencies SET currency_code = ? WHERE id = ?",
                (currency_code.upper(), currency_id)
            )
            conn.commit()
            return cursor.rowcount > 0
    
    def delete_currency(self, currency_id: int) -> bool:
        """
        Delete a currency.
        
        Args:
            currency_id: Currency ID
            
        Returns:
            True if currency was deleted, False if not found
        """
        with self._get_connection() as conn:
            cursor = conn.execute(
                "DELETE FROM currencies WHERE id = ?",
                (currency_id,)
            )
            conn.commit()
            return cursor.rowcount > 0
    
    def currency_exists(self, currency_code: str) -> bool:
        """
        Check if a currency exists by code.
        
        Args:
            currency_code: Currency code to check
            
        Returns:
            True if currency exists, False otherwise
        """
        with self._get_connection() as conn:
            cursor = conn.execute(
                "SELECT 1 FROM currencies WHERE currency_code = ?",
                (currency_code.upper(),)
            )
            return cursor.fetchone() is not None
    
    def get_currency_count(self) -> int:
        """
        Get the total number of currencies.
        
        Returns:
            Number of currencies
        """
        with self._get_connection() as conn:
            cursor = conn.execute("SELECT COUNT(*) FROM currencies")
            return cursor.fetchone()[0]
    
    # =================
    # ACCOUNT OPERATIONS
    # =================
    
    def create_account(self, currency_id: int, name: str, iban: Optional[str] = None) -> int:
        """
        Create a new account.
        
        Args:
            currency_id: ID of the currency for this account
            name: Display name for the account
            iban: Optional IBAN number
            
        Returns:
            The ID of the created account
            
        Raises:
            sqlite3.IntegrityError: If currency_id doesn't exist
        """
        with self._get_connection() as conn:
            cursor = conn.execute(
                "INSERT INTO accounts (currency_id, name, iban) VALUES (?, ?, ?)",
                (currency_id, name, iban)
            )
            conn.commit()
            return cursor.lastrowid
    
    def get_account_by_id(self, account_id: int) -> Optional[dict]:
        """
        Get an account by its ID.
        
        Args:
            account_id: Account ID
            
        Returns:
            Account data as dict or None if not found
        """
        with self._get_connection() as conn:
            cursor = conn.execute(
                "SELECT id, currency_id, name, iban FROM accounts WHERE id = ?",
                (account_id,)
            )
            row = cursor.fetchone()
            return dict(row) if row else None
    
    def get_account_by_id_with_currency(self, account_id: int) -> Optional[dict]:
        """
        Get an account by its ID with currency information.
        
        Args:
            account_id: Account ID
            
        Returns:
            Account data with currency info as dict or None if not found
        """
        with self._get_connection() as conn:
            cursor = conn.execute("""
                SELECT 
                    a.id, a.currency_id, a.name, a.iban,
                    c.currency_code
                FROM accounts a
                JOIN currencies c ON a.currency_id = c.id
                WHERE a.id = ?
            """, (account_id,))
            row = cursor.fetchone()
            return dict(row) if row else None
    
    def get_all_accounts(self) -> List[dict]:
        """
        Get all accounts.
        
        Returns:
            List of account data as dicts, sorted by name
        """
        with self._get_connection() as conn:
            cursor = conn.execute(
                "SELECT id, currency_id, name, iban FROM accounts ORDER BY name"
            )
            rows = cursor.fetchall()
            return [dict(row) for row in rows]
    
    def get_all_accounts_with_currency(self) -> List[dict]:
        """
        Get all accounts with currency information.
        
        Returns:
            List of account data with currency info as dicts, sorted by name
        """
        with self._get_connection() as conn:
            cursor = conn.execute("""
                SELECT 
                    a.id, a.currency_id, a.name, a.iban,
                    c.currency_code
                FROM accounts a
                JOIN currencies c ON a.currency_id = c.id
                ORDER BY a.name
            """)
            rows = cursor.fetchall()
            return [dict(row) for row in rows]
    
    def get_accounts_by_currency(self, currency_id: int) -> List[dict]:
        """
        Get all accounts for a specific currency.
        
        Args:
            currency_id: Currency ID
            
        Returns:
            List of account data as dicts
        """
        with self._get_connection() as conn:
            cursor = conn.execute(
                "SELECT id, currency_id, name, iban FROM accounts WHERE currency_id = ? ORDER BY name",
                (currency_id,)
            )
            rows = cursor.fetchall()
            return [dict(row) for row in rows]
    
    def update_account(self, account_id: int, currency_id: int, name: str, iban: Optional[str] = None) -> bool:
        """
        Update an account.
        
        Args:
            account_id: Account ID
            currency_id: New currency ID
            name: New name
            iban: New IBAN (optional)
            
        Returns:
            True if account was updated, False if not found
        """
        with self._get_connection() as conn:
            cursor = conn.execute(
                "UPDATE accounts SET currency_id = ?, name = ?, iban = ? WHERE id = ?",
                (currency_id, name, iban, account_id)
            )
            conn.commit()
            return cursor.rowcount > 0
    
    def delete_account(self, account_id: int) -> bool:
        """
        Delete an account.
        
        Args:
            account_id: Account ID
            
        Returns:
            True if account was deleted, False if not found
        """
        with self._get_connection() as conn:
            cursor = conn.execute(
                "DELETE FROM accounts WHERE id = ?",
                (account_id,)
            )
            conn.commit()
            return cursor.rowcount > 0
    
    def account_exists(self, account_id: int) -> bool:
        """
        Check if an account exists.
        
        Args:
            account_id: Account ID
            
        Returns:
            True if account exists, False otherwise
        """
        with self._get_connection() as conn:
            cursor = conn.execute(
                "SELECT 1 FROM accounts WHERE id = ?",
                (account_id,)
            )
            return cursor.fetchone() is not None
    
    def get_account_count(self) -> int:
        """
        Get the total number of accounts.
        
        Returns:
            Number of accounts
        """
        with self._get_connection() as conn:
            cursor = conn.execute("SELECT COUNT(*) FROM accounts")
            return cursor.fetchone()[0]
    
    # =================
    # USER OPERATIONS
    # =================
    
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