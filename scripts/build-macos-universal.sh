#!/bin/bash
# Build a universal macOS binary (Apple Silicon + Intel) for local development.
# Run this script from the repository root on macOS.
# If you get "Permission denied", make it executable with: chmod +x scripts/build-macos-universal.sh

npm run tauri:build -- --target universal-apple-darwin
