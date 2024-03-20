#!/usr/bin/env python3

import sqlite3
import os

db_path = os.getenv("DB_PATH", "hwapi.db")


def create_table():
    """Create a basic table with dummy data"""
    conn = sqlite3.connect(db_path)
    c = conn.cursor()

    c.execute(
        "CREATE TABLE IF NOT EXISTS devices (id INTEGER PRIMARY KEY, name TEXT)"
    )
    c.execute(
        """
        INSERT INTO devices (name) VALUES
        ('VT6651 WiFi Adapter, 802.11b'),
        ('Motherboard M2N68-AM SE2'),
        ('AMD Athlon(tm) Dual Core Processor 5400B')
        """
    )

    conn.commit()
    conn.close()


if __name__ == "__main__":
    create_table()
    print("Database initialized.")
