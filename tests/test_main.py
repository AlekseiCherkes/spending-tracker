"""Test the main module functionality."""

import pytest

import spending_tracker
from spending_tracker import __main__


class TestSpendingTrackerModule:
    """Test the spending_tracker module."""

    def test_module_imports(self):
        """Test that the module can be imported successfully."""
        assert spending_tracker is not None
        assert hasattr(spending_tracker, "__version__")
        assert hasattr(spending_tracker, "__author__")
        assert hasattr(spending_tracker, "__description__")

    def test_module_metadata(self):
        """Test module metadata is properly set."""
        assert spending_tracker.__version__ == "0.1.0"
        assert spending_tracker.__author__ == "Your Name"
        assert (
            spending_tracker.__description__
            == "A Telegram bot for personal spending tracking"
        )

    def test_main_function_exists(self):
        """Test that the main function exists and is callable."""
        assert hasattr(__main__, "main")
        assert callable(__main__.main)

    def test_main_module_structure(self):
        """Test that the __main__ module has the proper structure."""
        with open("spending_tracker/__main__.py", "r") as f:
            content = f.read()

        # Check that it has the expected structure
        assert 'if __name__ == "__main__":' in content
        assert "main()" in content
        assert "def main() -> None:" in content
        assert "from .bot import run_bot" in content


class TestPackageStructure:
    """Test package structure and organization."""

    def test_package_structure(self):
        """Test that the package has the expected structure."""
        import spending_tracker
        import spending_tracker.__main__

        # Test that these modules can be imported
        assert spending_tracker is not None
        assert spending_tracker.__main__ is not None

    def test_version_consistency(self):
        """Test that version is consistent across files."""
        # Version should be defined in __init__.py
        assert spending_tracker.__version__ == "0.1.0"


if __name__ == "__main__":
    # Allow running this test file directly
    pytest.main([__file__])
