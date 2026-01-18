#!/usr/bin/env python3
"""
Fetch ALL OpenHands documentation by fetching llms.txt from the web.

This script:
1. Fetches llms.txt from https://docs.openhands.dev/llms.txt
2. Extracts ALL URLs using regex
3. Fetches every single page
"""

import requests
import re
from pathlib import Path
from urllib.parse import urlparse
import time

LLMS_TXT_URL = "https://docs.openhands.dev/llms.txt"
OUTPUT_DIR = Path("/home/thomas/src/projects/orchestrator-project/crow/docs/openhands_book/docs")

OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

def fetch_llms_txt():
    """Fetch llms.txt from the web."""
    print(f"Fetching llms.txt from {LLMS_TXT_URL}")
    response = requests.get(LLMS_TXT_URL, timeout=30)
    response.raise_for_status()
    return response.text

def extract_urls(content):
    """Extract ALL URLs from llms.txt using regex."""
    # Match ALL markdown links: [Title](https://docs.openhands.dev/...)
    urls = re.findall(r'\[.*?\]\((https://docs\.openhands\.dev/[^)]+\.md)\)', content)
    return urls

def fetch_url(url):
    """Fetch a single URL and save the markdown."""
    try:
        response = requests.get(url, timeout=30)
        response.raise_for_status()
        
        # Extract filename from URL
        parsed = urlparse(url)
        path = parsed.path
        
        # Remove leading slash and create filename
        filename = path.lstrip('/')
        
        # Create subdirectories if needed
        filepath = OUTPUT_DIR / filename
        filepath.parent.mkdir(parents=True, exist_ok=True)
        
        # Save the content
        filepath.write_text(response.text, encoding="utf-8")
        return True, filename
        
    except Exception as e:
        return False, str(e)

def main():
    """Fetch all documentation pages."""
    # Fetch llms.txt
    content = fetch_llms_txt()
    print(f"✓ Got llms.txt ({len(content)} bytes)\n")
    
    # Extract ALL URLs
    urls = extract_urls(content)
    print(f"Found {len(urls)} pages to fetch\n")
    
    # Fetch each page
    fetched = 0
    failed = 0
    
    for i, url in enumerate(urls, 1):
        success, result = fetch_url(url)
        
        if success:
            print(f"[{i}/{len(urls)}] ✓ {result}")
            fetched += 1
        else:
            print(f"[{i}/{len(urls)}] ✗ {url}: {result}")
            failed += 1
        
        # Rate limiting
        time.sleep(0.2)
    
    print(f"\n✨ Done! Fetched {fetched} pages, {failed} failed")
    print(f"📁 Files saved to: {OUTPUT_DIR}")

if __name__ == "__main__":
    main()


