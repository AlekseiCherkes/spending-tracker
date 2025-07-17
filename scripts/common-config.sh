#!/bin/bash

# Common configuration for spending-tracker scripts
# Source this file in other scripts for consistent logging

# Logging functions for consistency
log_info() {
    echo "ℹ️  $1" >&2
}

log_warning() {
    echo "⚠️  $1" >&2
}

log_error() {
    echo "❌ $1" >&2
}

log_success() {
    echo "✅ $1" >&2
}
