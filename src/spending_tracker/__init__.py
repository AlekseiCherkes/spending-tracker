from telegram import Update
from telegram.ext import ApplicationBuilder, CommandHandler, ContextTypes

import os

async def hello(update: Update, context: ContextTypes.DEFAULT_TYPE) -> None:
    await update.message.reply_text(f'Hello {update.effective_user.first_name}')


async def help(update: Update, context: ContextTypes.DEFAULT_TYPE) -> None:
    await update.message.reply_text(f'This is help message')


def main():
    token = os.environ['ST_TOKEN']

    app = ApplicationBuilder().token(token).build()

    app.add_handler(CommandHandler("hello", hello))
    app.add_handler(CommandHandler("help", help))

    app.run_polling()
