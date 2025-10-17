#!/usr/bin/env python3
"""
Initialize Postgres schema using sql/postgres_init.sql.
Usage:
  python scripts/init_postgres.py --pg-url postgresql://user:pass@host/db
"""
import argparse
import pathlib
import sys

import psycopg2

ROOT = pathlib.Path(__file__).resolve().parents[1]
SCHEMA = ROOT / "sql" / "postgres_init.sql"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--pg-url", required=True, help="Postgres URL")
    args = ap.parse_args()

    sql = SCHEMA.read_text()

    conn = psycopg2.connect(args.pg_url)
    conn.autocommit = True
    with conn.cursor() as cur:
        cur.execute(sql)
    print("Initialized Postgres schema.")


if __name__ == "__main__":
    sys.exit(main())
