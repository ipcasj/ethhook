#!/usr/bin/env python3
"""
Seed the C admin API database with demo users
This script creates users directly in the SQLite database
"""

import sqlite3
import bcrypt
import uuid
import sys
from datetime import datetime


def create_demo_users(db_path):
    """Create demo users in the database"""
    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()

    # Check if users table exists
    cursor.execute("SELECT name FROM sqlite_master WHERE type='table' AND name='users'")
    if not cursor.fetchone():
        print("❌ Error: users table doesn't exist. Database not initialized.")
        conn.close()
        return False

    users_to_create = [
        {
            "email": "demo@ethhook.com",
            "password": "demo123",
            "is_admin": 0,
        },
        {
            "email": "admin@ethhook.io",
            "password": "SecureAdmin123!",
            "is_admin": 1,
        },
    ]

    for user_data in users_to_create:
        # Check if user already exists
        cursor.execute("SELECT id FROM users WHERE email = ?", (user_data["email"],))
        existing = cursor.fetchone()

        if existing:
            print(f"⚠️  User {user_data['email']} already exists, skipping...")
            continue

        # Generate user ID
        user_id = str(uuid.uuid4())

        # Hash password using bcrypt
        password_hash = bcrypt.hashpw(
            user_data["password"].encode("utf-8"), bcrypt.gensalt()
        ).decode("utf-8")

        # Get current timestamp in Unix epoch (seconds)
        created_at = int(datetime.now().timestamp())

        # Insert user using email column
        cursor.execute(
            """
            INSERT INTO users (id, email, password_hash, is_admin, created_at)
            VALUES (?, ?, ?, ?, ?)
        """,
            (
                user_id,
                user_data["email"],
                password_hash,
                user_data["is_admin"],
                created_at,
            ),
        )

        admin_status = "👑 Admin" if user_data["is_admin"] else "👤 User"
        print(
            f"✅ Created {admin_status}: {user_data['email']} (password: {user_data['password']})"
        )

    conn.commit()
    conn.close()
    return True


def main():
    if len(sys.argv) > 1:
        db_path = sys.argv[1]
    else:
        db_path = "/data/config.db"

    print(f"🔧 Seeding database: {db_path}")
    print()

    try:
        if create_demo_users(db_path):
            print()
            print("=" * 60)
            print("✅ Database seeded successfully!")
            print("=" * 60)
            print()
            print("You can now login with:")
            print()
            print("Demo User:")
            print("  Email:    demo@ethhook.com")
            print("  Password: demo123")
            print()
            print("Admin User:")
            print("  Email:    admin@ethhook.io")
            print("  Password: SecureAdmin123!")
            print()
        else:
            sys.exit(1)
    except Exception as e:
        print(f"❌ Error: {e}")
        import traceback

        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    main()
