"""Tests for the bot module."""

from spending_tracker import bot


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

    def test_dal_basic_operations(self):
        """Test that DAL operations work through the bot module."""
        # Test getting user count (should work even with empty database)
        count = bot.dal.get_user_count()
        assert isinstance(count, int)
        assert count >= 0

        # Test getting all users (should work even with empty database)
        users = bot.dal.get_all_users()
        assert isinstance(users, list)
