"""
Compute statistics for the data integration.

Usage:
    integration-stats.py
"""

from logging import getLogger

import duckdb
from bookdata.config import BookDataConfig
from bookdata.logging import setup_logging

_log = getLogger("integration-stats")

GENDER_FILE = "book-links/cluster-genders.parquet"
ISBN_CLUSTER_FILE = "book-links/isbn-clusters.parquet"
LOC_BOOK_FILE = "loc-mds/book-isbn-ids.parquet"
STAT_FILE = "book-links/gender-stats.csv"
ACTION_FILES = [
    ("BX-I", "bx/bx-cluster-actions.parquet"),
    ("BX-E", "bx/bx-cluster-ratings.parquet"),
    ("AZ14", "az2014/az-cluster-ratings.parquet"),
    ("AZ18", "az2018/az-cluster-ratings.parquet"),
    ("GR-I", "goodreads/gr-cluster-actions.parquet"),
    ("GR-E", "goodreads/gr-cluster-ratings.parquet"),
]


def main():
    setup_logging()
    cfg = BookDataConfig.load()

    with duckdb.connect() as db:
        db.execute(
            """
            CREATE TABLE gender_stats (
                dataset VARCHAR NOT NULL,
                gender VARCHAR NOT NULL,
                n_books INTEGER NOT NULL,
                n_actions INTEGER NULL
            )
            """
        )
        scan_genders(db)
        scan_loc(db)
        for name, file in ACTION_FILES:
            if cfg.source_enabled(name):
                scan_actions(db, name, file)

        _log.info("writing output")
        db.execute(f"COPY gender_stats TO '{STAT_FILE}'")


def scan_genders(db: duckdb.DuckDBPyConnection):
    _log.info("scanning %s", GENDER_FILE)
    db.execute(f"CREATE VIEW cluster_genders AS SELECT * FROM '{GENDER_FILE}'")


def scan_loc(db: duckdb.DuckDBPyConnection):
    _log.info("scanning LOC")
    query = f"""
        INSERT INTO gender_stats (dataset, gender, n_books)
        SELECT 'LOC-MDS', gender, COUNT(DISTINCT cluster)
        FROM '{LOC_BOOK_FILE}'
        JOIN '{ISBN_CLUSTER_FILE}' USING (isbn_id)
        JOIN cluster_genders USING (cluster)
        GROUP BY gender
    """
    db.execute(query)


def scan_actions(db: duckdb.DuckDBPyConnection, name: str, file: str):
    _log.info("scanning %s", name)
    query = f"""
        INSERT INTO gender_stats (dataset, gender, n_books, n_actions)
        SELECT '{name}', gender, COUNT(DISTINCT item_id), COUNT(item_id)
        FROM '{file}' a
        JOIN cluster_genders c ON a.item_id = c.cluster
        GROUP BY gender
    """
    db.execute(query)


if __name__ == "__main__":
    main()
