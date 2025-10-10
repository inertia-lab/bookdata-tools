#!/usr/bin/env python
"""
Collect ISBNs from across the integration.

Usage:
    collect-isbns.py -o OUT

Options:
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
    "AZ118": ("../az2018/ratings.parquet", ["asin"], "az2018"),
}


def main(opts: ParsedOptions):
    basic_logging()
    cfg = yaml.load(CFG_FILE.read_text(), yaml.SafeLoader)
    with duckdb.connect() as db:
        active = []
        db.execute(
            """
            CREATE TABLE isbn_rec (
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

        query = """
            WITH isbn_counts AS (
                PIVOT isbn_rec
                ON source
                USING coalesce(sum(nrecs), 0)::int
            )
            SELECT CAST(row_number() OVER (ORDER BY isbn) AS INT32) AS isbn_id, *
            FROM isbn_counts
        """
        file = opts["--output"]
        _log.info("writing result to %s", file)
        db.execute(
            f"""
            COPY ({query}) TO '{file}' (COMPRESSION zstd)
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
