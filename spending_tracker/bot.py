"""
Simple Telegram Bot for Spending Tracker
"""

import logging
import os

from dotenv import load_dotenv
from telegram import Update
from telegram.ext import Application, CommandHandler, ContextTypes

from .dal import SpendingTrackerDAL

# Load environment variables
load_dotenv()

# Set up logging
logging.basicConfig(
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s", level=logging.INFO
)
logger = logging.getLogger(__name__)

# Initialize DAL
dal = SpendingTrackerDAL()


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
            "❌ Sorry, there was an error retrieving the user list.\n"
            "Please try again later."
        )


async def unknown_command(update: Update, context: ContextTypes.DEFAULT_TYPE) -> None:
    """Handle unknown commands."""
    if not update.message:
        return

    await update.message.reply_text(
        "❓ Sorry, I didn't understand that command.\n\n"
        "Use /help to see available commands."
    )


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

    logger.info("✅ Bot commands registered")
    logger.info("💰 Spending Tracker Bot is ready!")

    # Run the bot until the user presses Ctrl-C
    application.run_polling(allowed_updates=Update.ALL_TYPES)


if __name__ == "__main__":
    run_bot()
