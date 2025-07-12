"""Tests for the bot module."""

from unittest.mock import AsyncMock, MagicMock, patch

import pytest
from telegram import InlineKeyboardMarkup, Update

from spending_tracker import bot
from spending_tracker.expense_state import DraftExpense, expense_state_manager


class TestBotModule:
    """Test the bot module."""

    def test_bot_module_imports(self):
        """Test that the bot module can be imported successfully."""
        assert bot is not None
        assert hasattr(bot, "run_bot")
        assert callable(bot.run_bot)

    def test_dal_initialization(self):
        """Test that the DAL is initialized in the bot module."""
        assert hasattr(bot, "dal")
        assert bot.dal is not None

        # Test that DAL methods are available
        assert hasattr(bot.dal, "get_user_count")
        assert hasattr(bot.dal, "get_all_users")
        assert callable(bot.dal.get_user_count)
        assert callable(bot.dal.get_all_users)

    def test_command_handlers_exist(self):
        """Test that command handler functions exist."""
        assert hasattr(bot, "start")
        assert hasattr(bot, "help_command")
        assert hasattr(bot, "status")
        assert hasattr(bot, "about")
        assert hasattr(bot, "users_command")

        # Test that they are callable
        assert callable(bot.start)
        assert callable(bot.help_command)
        assert callable(bot.status)
        assert callable(bot.about)
        assert callable(bot.users_command)

    def test_expense_handlers_exist(self):
        """Test that expense handler functions exist."""
        assert hasattr(bot, "handle_text_message")
        assert hasattr(bot, "handle_callback_query")
        assert hasattr(bot, "parse_amount_from_text")
        assert hasattr(bot, "ensure_user_exists")

        # Test that they are callable
        assert callable(bot.handle_text_message)
        assert callable(bot.handle_callback_query)
        assert callable(bot.parse_amount_from_text)
        assert callable(bot.ensure_user_exists)

    def test_dal_basic_operations(self):
        """Test that DAL operations work through the bot module."""
        # Test getting user count (should work even with empty database)
        count = bot.dal.get_user_count()
        assert isinstance(count, int)
        assert count >= 0

        # Test getting all users (should work even with empty database)
        users = bot.dal.get_all_users()
        assert isinstance(users, list)


class TestAmountParsing:
    """Test amount parsing functionality."""

    def test_parse_amount_from_text_integer(self):
        """Test parsing integer amounts."""
        assert bot.parse_amount_from_text("15") == 15.0
        assert bot.parse_amount_from_text("купил хлеб 20") == 20.0
        assert bot.parse_amount_from_text("потратил 100 рублей") == 100.0

    def test_parse_amount_from_text_decimal(self):
        """Test parsing decimal amounts."""
        assert bot.parse_amount_from_text("15.50") == 15.5
        assert bot.parse_amount_from_text("кофе 3.20") == 3.2
        assert bot.parse_amount_from_text("обед 12,75") == 12.75  # Comma as decimal separator

    def test_parse_amount_from_text_multiple_numbers(self):
        """Test parsing with multiple numbers (should take first)."""
        assert bot.parse_amount_from_text("купил 2 хлеба по 15 рублей") == 2.0
        assert bot.parse_amount_from_text("потратил 50.25 из 100") == 50.25

    def test_parse_amount_from_text_no_numbers(self):
        """Test parsing text without numbers."""
        assert bot.parse_amount_from_text("купил хлеб") is None
        assert bot.parse_amount_from_text("hello world") is None
        assert bot.parse_amount_from_text("") is None

    def test_parse_amount_from_text_zero_negative(self):
        """Test parsing zero and negative amounts."""
        assert bot.parse_amount_from_text("0") is None  # Zero should be None
        assert bot.parse_amount_from_text("0.00") is None
        # Note: regex doesn't capture negative numbers, so "-15" won't match


class TestKeyboardCreation:
    """Test keyboard creation functions."""

    def test_create_expense_keyboard_complete(self):
        """Test creating expense keyboard with complete data."""
        # Mock DAL responses
        with (
            patch.object(bot.dal, "get_category_by_id") as mock_get_category,
            patch.object(bot.dal, "get_account_by_id") as mock_get_account,
        ):

            mock_get_category.return_value = {"name": "Продукты"}
            mock_get_account.return_value = {"name": "Revolut"}

            draft = DraftExpense(amount=25.50, telegram_id=123456, category_id=1, account_id=1)

            keyboard = bot.create_expense_keyboard(draft)

            assert isinstance(keyboard, InlineKeyboardMarkup)
            assert len(keyboard.inline_keyboard) == 4  # Amount, Category, Account, Actions

            # Check that Save button is present (complete draft)
            action_buttons = keyboard.inline_keyboard[3]
            button_texts = [btn.text for btn in action_buttons]
            assert "✅ Сохранить" in button_texts
            assert "❌ Отменить" in button_texts

    def test_create_expense_keyboard_incomplete(self):
        """Test creating expense keyboard with incomplete data."""
        draft = DraftExpense(
            amount=25.50,
            telegram_id=123456,
            # Missing category_id and account_id
        )

        keyboard = bot.create_expense_keyboard(draft)

        assert isinstance(keyboard, InlineKeyboardMarkup)

        # Check that Save button is NOT present (incomplete draft)
        action_buttons = keyboard.inline_keyboard[3]
        button_texts = [btn.text for btn in action_buttons]
        assert "✅ Сохранить" not in button_texts
        assert "❌ Отменить" in button_texts

    def test_create_category_keyboard(self):
        """Test creating category selection keyboard."""
        with patch.object(bot.dal, "get_all_categories") as mock_get_categories:
            mock_get_categories.return_value = [{"id": 1, "name": "Продукты"}, {"id": 2, "name": "Транспорт"}]

            keyboard = bot.create_category_keyboard()

            assert isinstance(keyboard, InlineKeyboardMarkup)
            assert len(keyboard.inline_keyboard) == 3  # 2 categories + back button

            # Check category buttons
            assert keyboard.inline_keyboard[0][0].text == "Продукты"
            assert keyboard.inline_keyboard[0][0].callback_data == "select_category_1"
            assert keyboard.inline_keyboard[1][0].text == "Транспорт"
            assert keyboard.inline_keyboard[1][0].callback_data == "select_category_2"

            # Check back button
            assert keyboard.inline_keyboard[2][0].text == "← Назад"
            assert keyboard.inline_keyboard[2][0].callback_data == "back_to_expense"

    def test_create_account_keyboard(self):
        """Test creating account selection keyboard."""
        with patch.object(bot.dal, "get_all_accounts") as mock_get_accounts:
            mock_get_accounts.return_value = [{"id": 1, "name": "Revolut"}, {"id": 2, "name": "Nordea"}]

            keyboard = bot.create_account_keyboard()

            assert isinstance(keyboard, InlineKeyboardMarkup)
            assert len(keyboard.inline_keyboard) == 3  # 2 accounts + back button

            # Check account buttons
            assert keyboard.inline_keyboard[0][0].text == "Revolut"
            assert keyboard.inline_keyboard[0][0].callback_data == "select_account_1"
            assert keyboard.inline_keyboard[1][0].text == "Nordea"
            assert keyboard.inline_keyboard[1][0].callback_data == "select_account_2"

            # Check back button
            assert keyboard.inline_keyboard[2][0].text == "← Назад"
            assert keyboard.inline_keyboard[2][0].callback_data == "back_to_expense"


class TestUserManagement:
    """Test user management functions."""

    @pytest.mark.asyncio
    async def test_ensure_user_exists_new_user(self):
        """Test ensuring user exists for new user."""
        with (
            patch.object(bot.dal, "get_user_by_telegram_id") as mock_get_user,
            patch.object(bot.dal, "create_user") as mock_create_user,
        ):

            mock_get_user.return_value = None  # User doesn't exist
            mock_create_user.return_value = 123  # New user ID

            user_id = await bot.ensure_user_exists(telegram_id=456789, name="Test User")

            assert user_id == 123
            mock_get_user.assert_called_once_with(456789)
            mock_create_user.assert_called_once_with("Test User", 456789)

    @pytest.mark.asyncio
    async def test_ensure_user_exists_existing_user(self):
        """Test ensuring user exists for existing user."""
        with patch.object(bot.dal, "get_user_by_telegram_id") as mock_get_user:
            mock_get_user.return_value = {"id": 123, "name": "Existing User"}

            user_id = await bot.ensure_user_exists(telegram_id=456789, name="Test User")

            assert user_id == 123
            mock_get_user.assert_called_once_with(456789)

    @pytest.mark.asyncio
    async def test_get_default_category_id(self):
        """Test getting default category ID."""
        with patch.object(bot.dal, "get_all_categories") as mock_get_categories:
            mock_get_categories.return_value = [{"id": 1, "name": "Продукты"}, {"id": 2, "name": "Транспорт"}]

            category_id = await bot.get_default_category_id()

            assert category_id == 1
            mock_get_categories.assert_called_once()

    @pytest.mark.asyncio
    async def test_get_default_category_id_no_categories(self):
        """Test getting default category ID when no categories exist."""
        with patch.object(bot.dal, "get_all_categories") as mock_get_categories:
            mock_get_categories.return_value = []

            category_id = await bot.get_default_category_id()

            assert category_id is None

    @pytest.mark.asyncio
    async def test_get_default_account_id(self):
        """Test getting default account ID."""
        with patch.object(bot.dal, "get_all_accounts") as mock_get_accounts:
            mock_get_accounts.return_value = [{"id": 1, "name": "Revolut"}, {"id": 2, "name": "Nordea"}]

            account_id = await bot.get_default_account_id()

            assert account_id == 1
            mock_get_accounts.assert_called_once()

    @pytest.mark.asyncio
    async def test_get_default_account_id_no_accounts(self):
        """Test getting default account ID when no accounts exist."""
        with patch.object(bot.dal, "get_all_accounts") as mock_get_accounts:
            mock_get_accounts.return_value = []

            account_id = await bot.get_default_account_id()

            assert account_id is None


class TestCommandHandlers:
    """Test command handler functions."""

    def create_mock_update(self, text="/start", user_id=123, user_name="Test User"):
        """Helper to create mock Update object."""
        user = MagicMock()
        user.id = user_id
        user.first_name = user_name
        user.is_bot = False

        chat = MagicMock()
        chat.id = 456
        chat.type = "private"

        message = MagicMock()
        message.message_id = 1
        message.date = None
        message.chat = chat
        message.from_user = user
        message.text = text
        message.reply_text = AsyncMock()

        update = MagicMock()
        update.update_id = 1
        update.message = message
        update.effective_user = user

        return update

    @pytest.mark.asyncio
    async def test_start_command(self):
        """Test /start command handler."""
        update = self.create_mock_update("/start", user_name="John")
        context = MagicMock()

        await bot.start(update, context)

        # Check that reply was sent
        update.message.reply_text.assert_called_once()
        call_args = update.message.reply_text.call_args[0][0]
        assert "Hello John!" in call_args
        assert "spending tracker bot" in call_args

    @pytest.mark.asyncio
    async def test_help_command(self):
        """Test /help command handler."""
        update = self.create_mock_update("/help")
        context = MagicMock()

        await bot.help_command(update, context)

        # Check that reply was sent
        update.message.reply_text.assert_called_once()
        call_args = update.message.reply_text.call_args[0][0]
        assert "Commands:" in call_args
        assert "/start" in call_args
        assert "/help" in call_args

    @pytest.mark.asyncio
    async def test_status_command(self):
        """Test /status command handler."""
        update = self.create_mock_update("/status")
        context = MagicMock()

        with patch.object(bot.dal, "get_user_count") as mock_get_user_count:
            mock_get_user_count.return_value = 5

            await bot.status(update, context)

            # Check that reply was sent
            update.message.reply_text.assert_called_once()
            call_args = update.message.reply_text.call_args[0][0]
            assert "Bot is running!" in call_args
            assert "Registered Users: 5" in call_args

    @pytest.mark.asyncio
    async def test_start_command_no_message(self):
        """Test /start command with no message."""
        update = Update(update_id=1)  # No message
        context = MagicMock()

        # Should not raise an error
        await bot.start(update, context)

    @pytest.mark.asyncio
    async def test_start_command_no_user(self):
        """Test /start command with no user."""
        chat = MagicMock()
        chat.id = 456
        chat.type = "private"

        message = MagicMock()
        message.message_id = 1
        message.date = None
        message.chat = chat
        message.text = "/start"
        message.reply_text = AsyncMock()

        update = MagicMock()
        update.update_id = 1
        update.message = message
        update.effective_user = None

        context = MagicMock()

        await bot.start(update, context)

        # Should send error message
        update.message.reply_text.assert_called_once()
        call_args = update.message.reply_text.call_args[0][0]
        assert "couldn't identify you" in call_args


class TestTextMessageHandler:
    """Test text message handling for expenses."""

    def create_mock_update(self, text="купил кофе 3.50", user_id=123, user_name="Test User"):
        """Helper to create mock Update object."""
        user = MagicMock()
        user.id = user_id
        user.first_name = user_name
        user.is_bot = False

        chat = MagicMock()
        chat.id = 456
        chat.type = "private"

        message = MagicMock()
        message.message_id = 1
        message.date = None
        message.chat = chat
        message.from_user = user
        message.text = text
        message.reply_text = AsyncMock()

        update = MagicMock()
        update.update_id = 1
        update.message = message
        update.effective_user = user

        return update

    @pytest.mark.asyncio
    async def test_handle_text_message_with_amount(self):
        """Test handling text message with amount."""
        update = self.create_mock_update("купил кофе 3.50")
        context = MagicMock()

        with (
            patch.object(bot, "ensure_user_exists") as mock_ensure_user,
            patch.object(bot, "get_default_category_id") as mock_get_category,
            patch.object(bot, "get_default_account_id") as mock_get_account,
        ):

            mock_ensure_user.return_value = 123
            mock_get_category.return_value = 1
            mock_get_account.return_value = 1

            await bot.handle_text_message(update, context)

            # Check that user was ensured
            mock_ensure_user.assert_called_once_with(123, "Test User")

            # Check that reply was sent
            update.message.reply_text.assert_called_once()
            call_args = update.message.reply_text.call_args
            assert "💰 Новая трата: 3.50" in call_args[0][0]

            # Check that keyboard was provided
            assert "reply_markup" in call_args[1]
            assert isinstance(call_args[1]["reply_markup"], InlineKeyboardMarkup)

    @pytest.mark.asyncio
    async def test_handle_text_message_no_amount(self):
        """Test handling text message without amount."""
        update = self.create_mock_update("просто текст без цифр")
        context = MagicMock()

        await bot.handle_text_message(update, context)

        # Should not send any reply
        update.message.reply_text.assert_not_called()

    @pytest.mark.asyncio
    async def test_handle_text_message_no_defaults(self):
        """Test handling text message when no default category/account."""
        update = self.create_mock_update("купил кофе 3.50")
        context = MagicMock()

        with (
            patch.object(bot, "ensure_user_exists") as mock_ensure_user,
            patch.object(bot, "get_default_category_id") as mock_get_category,
            patch.object(bot, "get_default_account_id") as mock_get_account,
        ):

            mock_ensure_user.return_value = 123
            mock_get_category.return_value = None
            mock_get_account.return_value = None

            await bot.handle_text_message(update, context)

            # Should still send reply with keyboard
            update.message.reply_text.assert_called_once()
            call_args = update.message.reply_text.call_args
            assert "💰 Новая трата: 3.50" in call_args[0][0]

    @pytest.mark.asyncio
    async def test_handle_text_message_no_message(self):
        """Test handling when no message."""
        update = Update(update_id=1)  # No message
        context = MagicMock()

        # Should not raise an error
        await bot.handle_text_message(update, context)

    @pytest.mark.asyncio
    async def test_handle_text_message_no_user(self):
        """Test handling when no user."""
        chat = MagicMock()
        chat.id = 456
        chat.type = "private"

        message = MagicMock()
        message.message_id = 1
        message.date = None
        message.chat = chat
        message.text = "купил кофе 3.50"
        message.reply_text = AsyncMock()

        update = MagicMock()
        update.update_id = 1
        update.message = message
        update.effective_user = None

        context = MagicMock()

        await bot.handle_text_message(update, context)

        # Should not send any reply
        update.message.reply_text.assert_not_called()


class TestCallbackQueryHandler:
    """Test callback query handling for inline keyboards."""

    def create_mock_callback_query(
        self, callback_data="save_expense", user_id=123, user_name="Test User", message_id=1
    ):
        """Helper to create mock CallbackQuery object."""
        user = MagicMock()
        user.id = user_id
        user.first_name = user_name
        user.is_bot = False

        chat = MagicMock()
        chat.id = 456
        chat.type = "private"

        message = MagicMock()
        message.message_id = message_id
        message.date = None
        message.chat = chat
        message.from_user = user
        message.text = "Original message"
        message.edit_text = AsyncMock()
        message.edit_reply_markup = AsyncMock()

        callback_query = MagicMock()
        callback_query.id = "1"
        callback_query.from_user = user
        callback_query.chat_instance = "test"
        callback_query.data = callback_data
        callback_query.message = message
        callback_query.answer = AsyncMock()
        callback_query.edit_message_text = AsyncMock()
        callback_query.edit_message_reply_markup = AsyncMock()

        update = MagicMock()
        update.update_id = 1
        update.callback_query = callback_query
        update.effective_user = user

        return update

    @pytest.mark.asyncio
    async def test_handle_callback_query_save_expense(self):
        """Test handling save expense callback."""
        update = self.create_mock_callback_query("save_expense", message_id=1)
        context = MagicMock()

        # Setup draft expense
        expense_state_manager.create_draft(123, 25.50)
        expense_state_manager.update_draft(123, category_id=1, account_id=1, message_id=1)

        with (
            patch.object(bot, "ensure_user_exists") as mock_ensure_user,
            patch.object(bot.dal, "create_spending") as mock_create_spending,
        ):

            mock_ensure_user.return_value = 123
            mock_create_spending.return_value = 456

            await bot.handle_callback_query(update, context)

            # Check that spending was created
            mock_create_spending.assert_called_once()

            # Check that callback was answered
            update.callback_query.answer.assert_called_once()

            # Check that message was edited
            update.callback_query.edit_message_text.assert_called_once()

            # Check that draft was removed
            assert not expense_state_manager.has_draft(123)

    @pytest.mark.asyncio
    async def test_handle_callback_query_cancel_expense(self):
        """Test handling cancel expense callback."""
        update = self.create_mock_callback_query("cancel_expense", message_id=1)
        context = MagicMock()

        # Setup draft expense
        expense_state_manager.create_draft(123, 25.50)
        expense_state_manager.update_draft(123, message_id=1)

        await bot.handle_callback_query(update, context)

        # Check that callback was answered
        update.callback_query.answer.assert_called_once()

        # Check that message was edited
        update.callback_query.edit_message_text.assert_called_once()
        call_args = update.callback_query.edit_message_text.call_args[0][0]
        assert "❌ Трата отменена" in call_args

        # Check that draft was removed
        assert not expense_state_manager.has_draft(123)

    @pytest.mark.asyncio
    async def test_handle_callback_query_select_category(self):
        """Test handling category selection callback."""
        update = self.create_mock_callback_query("select_category_2", message_id=1)
        context = MagicMock()

        # Setup draft expense
        expense_state_manager.create_draft(123, 25.50)
        expense_state_manager.update_draft(123, message_id=1)

        await bot.handle_callback_query(update, context)

        # Check that callback was answered
        update.callback_query.answer.assert_called_once()

        # Check that message was edited with new keyboard
        update.callback_query.edit_message_text.assert_called_once()

        # Check that draft was updated
        updated_draft = expense_state_manager.get_draft(123)
        assert updated_draft.category_id == 2

    @pytest.mark.asyncio
    async def test_handle_callback_query_select_account(self):
        """Test handling account selection callback."""
        update = self.create_mock_callback_query("select_account_3", message_id=1)
        context = MagicMock()

        # Setup draft expense
        expense_state_manager.create_draft(123, 25.50)
        expense_state_manager.update_draft(123, message_id=1)

        await bot.handle_callback_query(update, context)

        # Check that callback was answered
        update.callback_query.answer.assert_called_once()

        # Check that message was edited with new keyboard
        update.callback_query.edit_message_text.assert_called_once()

        # Check that draft was updated
        updated_draft = expense_state_manager.get_draft(123)
        assert updated_draft.account_id == 3

    @pytest.mark.asyncio
    async def test_handle_callback_query_edit_category(self):
        """Test handling edit category callback."""
        update = self.create_mock_callback_query("edit_category", message_id=1)
        context = MagicMock()

        # Setup draft expense
        expense_state_manager.create_draft(123, 25.50)
        expense_state_manager.update_draft(123, message_id=1)

        await bot.handle_callback_query(update, context)

        # Check that callback was answered
        update.callback_query.answer.assert_called_once()

        # Check that message was edited with category keyboard
        update.callback_query.edit_message_text.assert_called_once()

    @pytest.mark.asyncio
    async def test_handle_callback_query_no_draft(self):
        """Test handling callback when no draft exists."""
        update = self.create_mock_callback_query("save_expense", message_id=1)
        context = MagicMock()

        # No draft exists
        await bot.handle_callback_query(update, context)

        # Should answer and edit message with error
        update.callback_query.answer.assert_called_once()
        update.callback_query.edit_message_text.assert_called_once()
        call_args = update.callback_query.edit_message_text.call_args
        assert "Сессия истекла" in call_args[0][0]

    @pytest.mark.asyncio
    async def test_handle_callback_query_no_callback_query(self):
        """Test handling when no callback query."""
        update = MagicMock()
        update.update_id = 1
        update.callback_query = None  # No callback query
        context = MagicMock()

        # Should not raise an error
        await bot.handle_callback_query(update, context)

    def teardown_method(self):
        """Clean up after each test."""
        # Clear all drafts
        expense_state_manager._drafts.clear()
