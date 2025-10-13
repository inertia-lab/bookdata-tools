#!/usr/bin/env python
"""
Collect ISBNs from across the integration.

Usage:
    collect-isbns.py [-d DB] -o OUT

Options:
    -d DB, --database=DB
        Open database DB [default: isbns.db].
    -o OUT, --output=OUT
        Write output to OUT.
"""

from pathlib import Path

import duckdb
import yaml
from docopt import ParsedOptions, docopt
from lenskit.logging import basic_logging, get_logger

_log = get_logger("collect-isbns")

CFG_FILE = Path("../config.yaml")
SOURCES = {
    "LOC": ("../loc-mds/book-isbns.parquet", ["isbn"], None),
    "OL": ("../openlibrary/edition-isbns.parquet", ["isbn"], None),
    "GR": ("../goodreads/gr-book-ids.parquet", ["isbn10", "isbn13", "asin"], "goodreads"),
    "BX": ("../bx/cleaned-ratings.csv", ["isbn"], "bx"),
    "AZ14": ("../az2014/ratings.parquet", ["asin"], "az2014"),
    "AZ18": ("../az2018/ratings.parquet", ["asin"], "az2018"),
}


def main(opts: ParsedOptions):
    basic_logging()
    cfg = yaml.load(CFG_FILE.read_text(), yaml.SafeLoader)
    with duckdb.connect(opts["--database"]) as db:
        active = []
        db.execute(
            """
            CREATE TEMPORARY TABLE isbn_rec (
                isbn VARCHAR NOT NULL,
                source VARCHAR NOT NULL,
                nrecs INTEGER NOT NULL
            )
            """
        )
        for name, attrs in SOURCES.items():
            path, cols, key = attrs
            if key and not cfg[key].get("enabled", True):
                _log.info("source %s disabled", name)
                continue

            scan_source(db, name, path, cols)
            active.append(name)

        _log.info("creating ISBN ID table")
        db.execute(
            """
            CREATE TABLE isbns AS
            SELECT row_number() OVER (ORDER BY isbn) AS isbn_id, isbn
            FROM (SELECT DISTINCT isbn FROM isbn_rec)
            """
        )
        (nrow,) = db.fetchone() or (0,)
        _log.info("generated IDs for %d ISBNs", nrow)

        _log.info("indexing ISBN table")
        db.execute("ALTER TABLE isbns ADD PRIMARY KEY (isbn_id)")
        db.execute("CREATE UNIQUE INDEX isbn_uq ON isbns (isbn)")

        db.execute(
            """
            CREATE TABLE isbn_counts AS
            WITH isbn_rc AS (
                SELECT isbn_id, source, nrecs
                FROM isbn_rec
                JOIN isbns USING (isbn)
            )
            PIVOT isbn_rc
            ON source
            USING COALESCE(SUM(nrecs), 0)::int
            """
        )

        file = opts["--output"]
        _log.info("writing result to %s", file)
        db.execute(
            f"""
            COPY isbn_counts TO '{file}' (COMPRESSION zstd)
            """
        )


def scan_source(db: duckdb.DuckDBPyConnection, name: str, path: str, cols: list[str]):
    _log.info("loading %s", name, path=path)
    for col in cols:
        _log.info("counting column %s of %s", col, name)
        db.execute(
            f"""
            INSERT INTO isbn_rec (isbn, source, nrecs)
            SELECT {col}, '{name}', COUNT(*)
            FROM '{path}'
            WHERE {col} IS NOT NULL
            GROUP BY {col}
            """
        )


if __name__ == "__main__":
    opts = docopt(__doc__ or "")
    main(opts)
