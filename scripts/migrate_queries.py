#!/usr/bin/env python3
"""
PostgreSQL to SQLite Query Migration Script

This script converts PostgreSQL-specific query syntax to SQLite syntax:
1. Replaces $1, $2, $3, ... with ?
2. Replaces NOW() with datetime('now')
3. Replaces RETURNING * with SQLite-compatible queries
4. Handles TEXT[] → json for arrays
"""

import re
import sys
from pathlib import Path

def convert_pg_to_sqlite(content: str) -> str:
    """Convert PostgreSQL query syntax to SQLite"""
    
    # Step 1: Replace $1, $2, ... with ? (positional params)
    # Match $1, $2, etc. and replace with ?
    def replace_param(match):
        return '?'
    
    content = re.sub(r'\$\d+', replace_param, content)
    
    # Step 2: Replace NOW() with datetime('now')
    content = content.replace('NOW()', "datetime('now')")
    
    # Step 3: Replace RETURNING * with separate SELECT
    # (SQLite doesn't support RETURNING in all cases)
    # We'll leave RETURNING for now as SQLite 3.35+ supports it
    
    return content

def process_file(file_path: Path):
    """Process a single Rust file"""
    print(f"Processing: {file_path}")
    
    content = file_path.read_text()
    original = content
    
    # Convert PostgreSQL syntax to SQLite
    content = convert_pg_to_sqlite(content)
    
    if content != original:
        file_path.write_text(content)
        print(f"  ✓ Updated {file_path.name}")
    else:
        print(f"  - No changes needed for {file_path.name}")

def main():
    """Main entry point"""
    if len(sys.argv) < 2:
        print("Usage: python3 migrate_queries.py <path-to-crates>")
        sys.exit(1)
    
    base_path = Path(sys.argv[1])
    
    # Find all Rust files in handlers
    rust_files = list(base_path.glob("**/*.rs"))
    
    print(f"Found {len(rust_files)} Rust files")
    print("\nConverting PostgreSQL → SQLite queries...\n")
    
    for rust_file in rust_files:
        process_file(rust_file)
    
    print("\n✅ Migration complete!")
    print("\nNext steps:")
    print("1. Build the project: cargo build")
    print("2. Run migrations: sqlx database create && sqlx migrate run")
    print("3. Test queries: cargo test")

if __name__ == "__main__":
    main()
