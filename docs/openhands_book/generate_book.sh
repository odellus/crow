#!/bin/bash

# Script to regenerate the OpenHands Documentation Book

# Define paths
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
INPUT_FILE="$ROOT_DIR/crow/docs/openhands_book/llms.txt"
PYTHON_SCRIPT="$ROOT_DIR/createopenhandsbook.py"

echo "OpenHands Documentation Book Generator"
echo "======================================"
echo ""

# Check if input file exists
if [ ! -f "$INPUT_FILE" ]; then
    echo "Error: Input file not found: $INPUT_FILE"
    echo "Please ensure the file exists and try again."
    exit 1
fi

# Check if Python script exists
if [ ! -f "$PYTHON_SCRIPT" ]; then
    echo "Error: Python script not found: $PYTHON_SCRIPT"
    echo "Please ensure the script exists and try again."
    exit 1
fi

echo "Input file: $INPUT_FILE"
echo "Output directory: $SCRIPT_DIR"
echo "Generation script: $PYTHON_SCRIPT"
echo ""

echo "Generating book structure..."
python3 "$PYTHON_SCRIPT"

if [ $? -eq 0 ]; then
    echo ""
    echo "Book generated successfully!"
    echo ""
    echo "To view the book, navigate to:"
    echo "  $SCRIPT_DIR"
    echo ""
    echo "Start with: $SCRIPT_DIR/index.md"
else
    echo ""
    echo "Error: Book generation failed."
    exit 1
fi
