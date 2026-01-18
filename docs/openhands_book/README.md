# OpenHands Documentation Book

This directory contains the OpenHands Documentation Book, a structured organization of the OpenHands documentation extracted from the `llms.txt` file.

## Overview

The book is organized into sections and individual markdown files, making it easy to navigate and read the documentation in a book-like format.

## Directory Structure

```
openhands_book/
├── index.md                          # Main index file
├── docs/
│   ├── index.md                      # Docs section index
│   ├── add-event.md                  # Individual documentation entries
│   ├── alive.md
│   ├── ...
├── optional/
│   ├── index.md                      # Optional section index
│   ├── company.md
│   ├── blog.md
│   └── ...
```

## Usage

### Navigation

1. Start with the [main index file](index.md)
2. Browse through sections to find the topics you're interested in
3. Click on individual entries to read the detailed documentation

### Reading

Each entry contains:
- **Title**: The name of the documentation entry
- **URL**: The original OpenHands documentation URL
- **Description**: Content extracted from the llms.txt file
- **Source information**: Indicates this is part of the generated book

## Sections

### Docs

The main documentation section containing API references, guides, and tutorials organized by category.

### Optional

Additional resources including company information, blog posts, and external links.

## Generation Script

The book was generated using `createopenhandsbook.py`. This script:
- Parses the `llms.txt` file
- Extracts sections and entries
- Creates a hierarchical directory structure
- Generates individual markdown files
- Builds navigation indexes

To regenerate the book, run:
```bash
python createopenhandsbook.py
```

## Contributing

To update this book:
1. Edit the source `llms.txt` file
2. Regenerate the book using the script
3. Commit both the updated source and generated files

## License

This book is generated from the OpenHands documentation, which is available under the OpenHands project license.