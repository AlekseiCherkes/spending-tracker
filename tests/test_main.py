"""Test the main module functionality."""

import pytest
import sys
from io import StringIO
from unittest.mock import patch

import spending_tracker
from spending_tracker import __main__


class TestSpendingTrackerModule:
    """Test the spending_tracker module."""

    def test_module_imports(self):
        """Test that the module can be imported successfully."""
        assert spending_tracker is not None
        assert hasattr(spending_tracker, '__version__')
        assert hasattr(spending_tracker, '__author__')
        assert hasattr(spending_tracker, '__description__')

    def test_module_metadata(self):
        """Test module metadata is properly set."""
        assert spending_tracker.__version__ == "0.1.0"
        assert spending_tracker.__author__ == "Your Name"
        assert spending_tracker.__description__ == "A simple spending tracker application"

    def test_main_function_exists(self):
        """Test that the main function exists and is callable."""
        assert hasattr(__main__, 'main')
        assert callable(__main__.main)

    def test_main_function_output(self, capsys):
        """Test that the main function produces expected output."""
        # Call the main function
        __main__.main()
        
        # Capture the output
        captured = capsys.readouterr()
        
        # Check that expected strings are in the output
        assert "Hello World! 🌍" in captured.out
        assert "Welcome to Spending Tracker!" in captured.out
        assert "Version: 0.1.0" in captured.out
        assert "Author: Your Name" in captured.out
        assert "More features coming soon... 🚀" in captured.out

    def test_main_function_runs_without_error(self):
        """Test that the main function runs without raising exceptions."""
        try:
            __main__.main()
        except Exception as e:
            pytest.fail(f"main() raised {type(e).__name__}: {e}")

    def test_main_module_name_check(self):
        """Test that the __main__ module has the right name check."""
        # Test that the __main__.py file has the proper structure
        with open('spending_tracker/__main__.py', 'r') as f:
            content = f.read()
        
        # Check that it has the expected structure
        assert 'if __name__ == "__main__":' in content
        assert 'main()' in content
        assert 'def main():' in content


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
        
        # Could also check pyproject.toml if we parse it, but for now this is sufficient

    def test_main_function_content(self):
        """Test that the main function contains expected content."""
        # Test that main function would produce the expected output
        # This is a more reliable test than trying to exec the module
        import io
        import sys
        
        # Capture output
        captured_output = io.StringIO()
        sys.stdout = captured_output
        
        try:
            __main__.main()
            output = captured_output.getvalue()
            
            # Check for expected content
            assert "Hello World!" in output
            assert "Welcome to Spending Tracker!" in output
            assert "Version: 0.1.0" in output
        finally:
            sys.stdout = sys.__stdout__


if __name__ == "__main__":
    # Allow running this test file directly
    pytest.main([__file__]) 