#!/usr/bin/env python3
"""
Seed wallets from an existing SQLite database (wallets.db) into Postgres.

Assumes SQLite table:
  wallets(wallet TEXT PRIMARY KEY, alias/name TEXT, is_tracked INT, notes TEXT, created_at INT)

Usage:
  python scripts/seed_wallets.py \
      --sqlite /path/to/wallets.db \
      --pg-url postgresql://user:pass@host:5432/copytrader
"""
import argparse
import sqlite3
import sys
from typing import Iterable, Tuple

import psycopg2


def rows(sqlite_path: str) -> Iterable[Tuple[str, str, int, str]]:
    conn = sqlite3.connect(sqlite_path)
    cur = conn.cursor()
    # Check actual columns
    cur.execute("PRAGMA table_info(wallets)")
    cols = [r[1] for r in cur.fetchall()]

    # Map source columns to target
    wallet_col = "wallet_address" if "wallet_address" in cols else "wallet"
    label_col = "name" if "name" in cols else ("alias" if "alias" in cols else None)

    if label_col is None:
        raise SystemExit("wallets table must have 'name' or 'alias' column")

    # Select available columns, defaulting missing ones
    cur.execute(
        f"SELECT {wallet_col}, {label_col}, 1 as is_tracked, '' as notes FROM wallets"
    )
    for w, a, t, n in cur.fetchall():
        yield w, a, int(t), n


def upsert_postgres(pg_url: str, data: Iterable[Tuple[str, str, int, str]]):
    conn = psycopg2.connect(pg_url)
    conn.autocommit = True
    with conn.cursor() as cur:
        for wallet, alias, is_tracked, notes in data:
            cur.execute(
                """
                INSERT INTO wallets(wallet, alias, is_tracked, notes)
                VALUES (%s, %s, %s, %s)
                ON CONFLICT (wallet)
                DO UPDATE SET alias = EXCLUDED.alias,
                              is_tracked = EXCLUDED.is_tracked,
                              notes = EXCLUDED.notes
                """,
                (wallet, alias, bool(is_tracked), notes),
            )
    print("Seeded wallets into Postgres.")


def wallet_roles(sqlite_path: str):
    conn = sqlite3.connect(sqlite_path)
    cur = conn.cursor()
    try:
        # Check column name
        cur.execute("PRAGMA table_info(wallet_roles)")
        cols = [r[1] for r in cur.fetchall()]
        wallet_col = "wallet_address" if "wallet_address" in cols else "wallet"

        cur.execute(f"SELECT {wallet_col}, role FROM wallet_roles")
        return cur.fetchall()
    except sqlite3.OperationalError:
        return []


def upsert_wallet_roles(pg_url: str, data):
    conn = psycopg2.connect(pg_url)
    conn.autocommit = True
    with conn.cursor() as cur:
        for wallet, role in data:
            cur.execute(
                """
                INSERT INTO wallet_roles(wallet, role)
                VALUES (%s, %s)
                ON CONFLICT (wallet, role) DO NOTHING
                """,
                (wallet, role),
            )
    print("Seeded wallet_roles into Postgres.")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--sqlite", required=True, help="Path to source wallets.db (SQLite)"
    )
    ap.add_argument("--pg-url", required=True, help="Postgres connection URL")
    args = ap.parse_args()

    data = list(rows(args.sqlite))
    upsert_postgres(args.pg_url, data)

    roles = wallet_roles(args.sqlite)
    if roles:
        upsert_wallet_roles(args.pg_url, roles)


if __name__ == "__main__":
    sys.exit(main())
