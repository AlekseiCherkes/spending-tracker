"""
Main entry point for the spending tracker module.
This file allows the module to be run as: python -m spending_tracker
"""

from .bot import run_bot


def main():
    """Main function - entry point for the application"""
    run_bot()


if __name__ == "__main__":
    main() 