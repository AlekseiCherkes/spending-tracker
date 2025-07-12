#!/usr/bin/env python3
"""
Script to populate the database with test data.
Run this script to add sample currencies, users, categories, and accounts.
"""

import random
import sys
from pathlib import Path

# Add the project root to Python path
project_root = Path(__file__).parent.parent
sys.path.insert(0, str(project_root))

from spending_tracker.dal import SpendingTrackerDAL  # noqa: E402


def populate_test_data():
    """Populate the database with test data."""
    print("🚀 Starting test data population...")

    # Initialize DAL
    dal = SpendingTrackerDAL()

    # =================
    # CURRENCIES
    # =================
    print("\n💰 Adding currencies...")
    currencies = ["EUR", "USD", "BYN"]

    for currency_code in currencies:
        if not dal.currency_exists(currency_code):
            currency_id = dal.create_currency(currency_code)
            print(f"✅ Created currency: {currency_code} (ID: {currency_id})")
        else:
            print(f"⏩ Currency {currency_code} already exists")

    # =================
    # USERS
    # =================
    print("\n👥 Adding users...")
    users_data = [
        {"name": "Alex", "telegram_id": 5033919666},
        {"name": "Hanna", "telegram_id": random.randint(1000000000, 9999999999)},
    ]

    for user_data in users_data:
        if not dal.user_exists(user_data["telegram_id"]):
            user_id = dal.create_user(user_data["name"], user_data["telegram_id"])
            print(
                f"✅ Created user: {user_data['name']} "
                f"(Telegram ID: {user_data['telegram_id']}, DB ID: {user_id})"
            )
        else:
            print(
                f"⏩ User {user_data['name']} "
                f"(Telegram ID: {user_data['telegram_id']}) already exists"
            )

    # =================
    # CATEGORIES
    # =================
    print("\n📊 Adding categories...")
    categories = [
        "Продукты и хозтовары",
        "Еда вне дома",
        "Кофе и вкусняшки",
        "Развлечения и отдых",
        "Одежда",
        "Здоровье и медицина",
        "Спорт, забота о себе",
        "Образование",
        "Путешествия, туризм",
        "Дети (образование)",
        "Дети (хобби)",
        "Дети (присмотр)",
        "Интернет подписки",
        "Транспорт",
        "Автомобиль",
        "Автомобиль (аренда)",
        "Автомобиль (бензин, паркинг)",
        "Жильё",
        "Жильё (обустройство)",
        "Другое",
    ]

    for i, category_name in enumerate(categories):
        if not dal.category_exists(category_name):
            category_id = dal.create_category(category_name, sort_order=i)
            print(
                f"✅ Created category: {category_name} "
                f"(Sort order: {i}, ID: {category_id})"
            )
        else:
            print(f"⏩ Category '{category_name}' already exists")

    # =================
    # ACCOUNTS
    # =================
    print("\n🏦 Adding accounts...")

    # Get currency IDs
    eur_currency = dal.get_currency_by_code("EUR")
    if not eur_currency:
        print("❌ Error: EUR currency not found!")
        return

    # Get user IDs
    alex_user = dal.get_user_by_telegram_id(5033919666)
    hanna_user = dal.get_user_by_telegram_id(users_data[1]["telegram_id"])

    if not alex_user or not hanna_user:
        print("❌ Error: Users not found!")
        return

    accounts_data = [
        {
            "name": "Revolut (Joint)",
            "iban": "LT41 3250 0685 7871 5897",
            "currency_id": eur_currency["id"],
            "owner": "Alex",
        },
        {
            "name": "Nordea (Spending)",
            "iban": "FI90 1432 3500 6670 50",
            "currency_id": eur_currency["id"],
            "owner": "Alex",
        },
        {
            "name": "S-pankki",
            "iban": "FI90 1432 3500 6670 50",
            "currency_id": eur_currency["id"],
            "owner": "Hanna",
        },
    ]

    for account_data in accounts_data:
        # Check if account already exists by looking for same name
        existing_accounts = dal.get_all_accounts()
        account_exists = any(
            acc["name"] == account_data["name"] for acc in existing_accounts
        )

        if not account_exists:
            account_id = dal.create_account(
                currency_id=account_data["currency_id"],
                name=account_data["name"],
                iban=account_data["iban"],
            )
            print(
                f"✅ Created account: {account_data['name']} "
                f"(IBAN: {account_data['iban']}, "
                f"Owner: {account_data['owner']}, ID: {account_id})"
            )
        else:
            print(f"⏩ Account '{account_data['name']}' already exists")

    # =================
    # SUMMARY
    # =================
    print("\n📋 Summary:")
    print(f"💰 Total currencies: {dal.get_currency_count()}")
    print(f"👥 Total users: {dal.get_user_count()}")
    print(f"📊 Total categories: {dal.get_category_count()}")
    print(f"🏦 Total accounts: {dal.get_account_count()}")

    print("\n✅ Test data population completed!")


if __name__ == "__main__":
    try:
        populate_test_data()
    except Exception as e:
        print(f"❌ Error during test data population: {e}")
        sys.exit(1)
