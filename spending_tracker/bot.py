"""
Simple Telegram Bot for Spending Tracker
"""

import logging
import os
import re
from typing import Optional

from dotenv import load_dotenv
from telegram import InlineKeyboardButton, InlineKeyboardMarkup, Update
from telegram.ext import Application, CallbackQueryHandler, CommandHandler, ContextTypes, MessageHandler, filters

from .dal import SpendingTrackerDAL
from .expense_state import DraftExpense, expense_state_manager

# Load environment variables
load_dotenv()

# Set up logging
logging.basicConfig(format="%(asctime)s - %(name)s - %(levelname)s - %(message)s", level=logging.INFO)
logger = logging.getLogger(__name__)

# Initialize DAL
dal = SpendingTrackerDAL()


async def ensure_user_exists(telegram_id: int, name: str) -> int:
    """Ensure user exists in database and return user ID."""
    user = dal.get_user_by_telegram_id(telegram_id)
    if user:
        return int(user["id"])

    # Create new user
    user_id = dal.create_user(name, telegram_id)
    logger.info(f"Created new user: {name} (Telegram ID: {telegram_id}, DB ID: {user_id})")
    return user_id


def parse_amount_from_text(text: str) -> Optional[float]:
    """Parse amount from text message."""
    # Look for numbers (including decimals) in the text
    pattern = r"\d+(?:[.,]\d+)?"
    matches = re.findall(pattern, text)

    if matches:
        # Take the first number found
        amount_str = matches[0].replace(",", ".")
        try:
            amount = float(amount_str)
            return amount if amount > 0 else None
        except ValueError:
            return None

    return None


async def get_default_category_id() -> Optional[int]:
    """Get the default category (first by sort order)."""
    categories = dal.get_all_categories()
    if categories:
        return int(categories[0]["id"])  # First category by sort order
    return None


async def get_default_account_id() -> Optional[int]:
    """Get the default account (first available)."""
    accounts = dal.get_all_accounts()
    if accounts:
        return int(accounts[0]["id"])  # First account
    return None


def create_expense_keyboard(draft: DraftExpense) -> InlineKeyboardMarkup:
    """Create inline keyboard for expense editing."""
    keyboard = []

    # Amount row
    keyboard.append([InlineKeyboardButton(f"💰 Сумма: {draft.amount:.2f}", callback_data="edit_amount")])

    # Category row
    if draft.category_id:
        category = dal.get_category_by_id(draft.category_id)
        category_name = category["name"] if category else "Не выбрана"
    else:
        category_name = "Не выбрана"

    keyboard.append([InlineKeyboardButton(f"📊 Категория: {category_name}", callback_data="edit_category")])

    # Account row
    if draft.account_id:
        account = dal.get_account_by_id(draft.account_id)
        account_name = account["name"] if account else "Не выбран"
    else:
        account_name = "Не выбран"

    keyboard.append([InlineKeyboardButton(f"🏦 Счет: {account_name}", callback_data="edit_account")])

    # Action buttons
    action_row = []
    if draft.is_complete():
        action_row.append(InlineKeyboardButton("✅ Сохранить", callback_data="save_expense"))

    action_row.append(InlineKeyboardButton("❌ Отменить", callback_data="cancel_expense"))

    keyboard.append(action_row)

    return InlineKeyboardMarkup(keyboard)


def create_category_keyboard() -> InlineKeyboardMarkup:
    """Create inline keyboard for category selection."""
    categories = dal.get_all_categories()
    keyboard = []

    for category in categories:
        keyboard.append([InlineKeyboardButton(category["name"], callback_data=f"select_category_{category['id']}")])

    # Back button
    keyboard.append([InlineKeyboardButton("← Назад", callback_data="back_to_expense")])

    return InlineKeyboardMarkup(keyboard)


def create_account_keyboard() -> InlineKeyboardMarkup:
    """Create inline keyboard for account selection."""
    accounts = dal.get_all_accounts()
    keyboard = []

    for account in accounts:
        keyboard.append([InlineKeyboardButton(account["name"], callback_data=f"select_account_{account['id']}")])

    # Back button
    keyboard.append([InlineKeyboardButton("← Назад", callback_data="back_to_expense")])

    return InlineKeyboardMarkup(keyboard)


async def start(update: Update, context: ContextTypes.DEFAULT_TYPE) -> None:
    """Send a message when the command /start is issued."""
    if not update.message:
        return

    user = update.effective_user
    if not user:
        await update.message.reply_text("Sorry, I couldn't identify you.")
        return

    await update.message.reply_text(
        f"Hello {user.first_name}! 👋\n\n"
        "I'm your personal spending tracker bot! 💰\n\n"
        "I can help you:\n"
        "💳 Track your expenses\n"
        "📊 View spending summaries\n"
        "📈 Analyze your spending habits\n\n"
        "Use /help to see all available commands."
    )


async def help_command(update: Update, context: ContextTypes.DEFAULT_TYPE) -> None:
    """Send a message when the command /help is issued."""
    if not update.message:
        return

    help_text = """
🤖 **Spending Tracker Bot Commands:**

/start - Welcome message and introduction
/help - Show this help message
/status - Check bot status
/about - About this bot
/users - List all registered users

💡 **Coming Soon:**
• Add expenses
• View expense history
• Monthly summaries
• Category analytics

This is version 0.1.0 - more features coming soon! 🚀
    """
    await update.message.reply_text(help_text, parse_mode="Markdown")


async def status(update: Update, context: ContextTypes.DEFAULT_TYPE) -> None:
    """Show bot status."""
    if not update.message:
        return

    user_count = dal.get_user_count()
    await update.message.reply_text(
        "✅ Bot is running!\n"
        "🔋 Status: Online\n"
        "📱 Version: 0.1.0\n"
        "🐍 Python: 3.13\n"
        f"👥 Registered Users: {user_count}\n\n"
        "Ready to track your expenses! 💰"
    )


async def about(update: Update, context: ContextTypes.DEFAULT_TYPE) -> None:
    """Show information about the bot."""
    if not update.message:
        return

    about_text = """
📱 **Spending Tracker Bot**

🎯 **Purpose:** Help you track personal expenses
🏗️ **Built with:** Python 3.13 + python-telegram-bot
📦 **Version:** 0.1.0
⚡ **Status:** In Development

🔧 **Features:**
• Simple and intuitive interface
• Real-time expense tracking
• Personal data privacy
• Modern Python architecture

🚀 **Roadmap:**
• Expense categories
• Monthly reports
• Data visualization
• Export capabilities

Made with ❤️ using modern Python practices!
    """
    await update.message.reply_text(about_text, parse_mode="Markdown")


async def users_command(update: Update, context: ContextTypes.DEFAULT_TYPE) -> None:
    """List all registered users."""
    if not update.message:
        return

    try:
        users = dal.get_all_users()

        if not users:
            await update.message.reply_text(
                "👥 **Registered Users:**\n\n"
                "No users registered yet.\n"
                "Users will appear here when they start using the bot.",
                parse_mode="Markdown",
            )
            return

        # Format users list
        user_list = "👥 **Registered Users:**\n\n"
        for i, user in enumerate(users, 1):
            user_list += f"{i}. **{user['name']}**\n"
            user_list += f"   • ID: {user['id']}\n"
            user_list += f"   • Telegram ID: {user['telegram_id']}\n\n"

        user_list += f"📊 **Total Users:** {len(users)}"

        await update.message.reply_text(user_list, parse_mode="Markdown")

    except Exception as e:
        logger.error(f"Error retrieving users: {e}")
        await update.message.reply_text(
            "❌ Sorry, there was an error retrieving the user list.\n" "Please try again later."
        )


async def unknown_command(update: Update, context: ContextTypes.DEFAULT_TYPE) -> None:
    """Handle unknown commands."""
    if not update.message:
        return

    await update.message.reply_text(
        "❓ Sorry, I didn't understand that command.\n\n" "Use /help to see available commands."
    )


async def handle_text_message(update: Update, context: ContextTypes.DEFAULT_TYPE) -> None:
    """Handle text messages and check for amounts."""
    if not update.message or not update.effective_user or not update.message.text:
        return

    text = update.message.text
    user = update.effective_user
    telegram_id = user.id

    # Parse amount from message
    amount = parse_amount_from_text(text)

    if amount is None:
        return  # Not a spending message

    # Ensure user exists
    user_name = user.first_name or user.username or "Unknown"
    await ensure_user_exists(telegram_id, user_name)

    # Create draft expense
    draft = expense_state_manager.create_draft(telegram_id, amount)

    # Set defaults
    draft.category_id = await get_default_category_id()
    draft.account_id = await get_default_account_id()

    # Create keyboard
    keyboard = create_expense_keyboard(draft)

    # Send message with keyboard
    message_text = f"💰 Новая трата: {amount:.2f}\n\nВыберите параметры:"

    sent_message = await update.message.reply_text(message_text, reply_markup=keyboard)

    # Store message ID for editing
    draft.message_id = sent_message.message_id


async def handle_callback_query(update: Update, context: ContextTypes.DEFAULT_TYPE) -> None:
    """Handle callback queries from inline keyboards."""
    query = update.callback_query
    if not query or not query.from_user or not query.data:
        return

    await query.answer()

    telegram_id = query.from_user.id
    data = query.data

    draft = expense_state_manager.get_draft(telegram_id)
    if not draft:
        await query.edit_message_text("❌ Сессия истекла. Отправьте новую сумму.")
        return

    if data == "edit_category":
        # Show category selection
        keyboard = create_category_keyboard()
        await query.edit_message_text(
            f"💰 Трата: {draft.amount:.2f}\n\n📊 Выберите категорию:",
            reply_markup=keyboard,
        )

    elif data == "edit_account":
        # Show account selection
        keyboard = create_account_keyboard()
        await query.edit_message_text(
            f"💰 Трата: {draft.amount:.2f}\n\n🏦 Выберите счет:",
            reply_markup=keyboard,
        )

    elif data.startswith("select_category_"):
        # Category selected
        category_id = int(data.split("_")[2])
        draft.category_id = category_id

        # Go back to main expense view
        keyboard = create_expense_keyboard(draft)
        await query.edit_message_text(
            f"💰 Новая трата: {draft.amount:.2f}\n\nВыберите параметры:",
            reply_markup=keyboard,
        )

    elif data.startswith("select_account_"):
        # Account selected
        account_id = int(data.split("_")[2])
        draft.account_id = account_id

        # Go back to main expense view
        keyboard = create_expense_keyboard(draft)
        await query.edit_message_text(
            f"💰 Новая трата: {draft.amount:.2f}\n\nВыберите параметры:",
            reply_markup=keyboard,
        )

    elif data == "back_to_expense":
        # Go back to main expense view
        keyboard = create_expense_keyboard(draft)
        await query.edit_message_text(
            f"💰 Новая трата: {draft.amount:.2f}\n\nВыберите параметры:",
            reply_markup=keyboard,
        )

    elif data == "save_expense":
        # Save expense
        if not draft.is_complete():
            await query.edit_message_text("❌ Не все поля заполнены!")
            return

        # Type guard - at this point we know draft is complete
        if draft.account_id is None or draft.category_id is None:
            await query.edit_message_text("❌ Не все поля заполнены!")
            return

        # Get user ID
        user_name = query.from_user.first_name or query.from_user.username or "Unknown"
        user_id = await ensure_user_exists(telegram_id, user_name)

        # Save to database
        try:
            spending_id = dal.create_spending(
                account_id=draft.account_id,
                amount=draft.amount,
                category_id=draft.category_id,
                reporter_id=user_id,
                notes=draft.notes,
                timestamp=draft.timestamp,
            )

            # Get details for confirmation
            spending = dal.get_spending_by_id_with_details(spending_id)

            if spending:
                message_text = (
                    f"✅ Трата сохранена!\n\n"
                    f"💰 Сумма: {spending['amount']:.2f} {spending['currency_code']}\n"
                    f"📊 Категория: {spending['category_name']}\n"
                    f"🏦 Счет: {spending['account_name']}\n"
                    f"📅 Дата: {spending['timestamp']}"
                )
                await query.edit_message_text(message_text)
            else:
                await query.edit_message_text("✅ Трата сохранена!")

            # Remove draft
            expense_state_manager.remove_draft(telegram_id)

        except Exception as e:
            logger.error(f"Error saving expense: {e}")
            await query.edit_message_text("❌ Ошибка при сохранении траты.")

    elif data == "cancel_expense":
        # Cancel expense
        expense_state_manager.remove_draft(telegram_id)
        await query.edit_message_text("❌ Трата отменена.")


def run_bot() -> None:
    """Run the bot."""
    # Get bot token from environment
    token = os.getenv("TELEGRAM_BOT_TOKEN")

    if not token:
        logger.error("❌ TELEGRAM_BOT_TOKEN not found in environment variables!")
        logger.error("Please create a .env file with your bot token:")
        logger.error("TELEGRAM_BOT_TOKEN=your_token_here")
        return

    logger.info("🚀 Starting Spending Tracker Bot...")

    # Create the Application
    application = Application.builder().token(token).build()

    # Add command handlers
    application.add_handler(CommandHandler("start", start))
    application.add_handler(CommandHandler("help", help_command))
    application.add_handler(CommandHandler("status", status))
    application.add_handler(CommandHandler("about", about))
    application.add_handler(CommandHandler("users", users_command))

    # Add message handler for text messages
    application.add_handler(MessageHandler(filters.TEXT & ~filters.COMMAND, handle_text_message))

    # Add callback query handler for inline keyboards
    application.add_handler(CallbackQueryHandler(handle_callback_query))

    logger.info("✅ Bot commands registered")
    logger.info("💰 Spending Tracker Bot is ready!")

    # Run the bot until the user presses Ctrl-C
    application.run_polling(allowed_updates=Update.ALL_TYPES)


if __name__ == "__main__":
    run_bot()
