"""
Cluster the book graph data.

Usage:
    compute-clusters.py [-v] DB

Args:
    -v, --verbose
        Enable verbose log output.
    DB
        The name of the database file to save the clustering into.
"""

from pathlib import Path

import duckdb
from bookdata.codes import register_udfs
from bookdata.config import BookDataConfig
from docopt import docopt
from lenskit.logging import LoggingConfig, get_logger

_log = get_logger("cluster")


def main():
    opts = docopt(__doc__ or "")
    lc = LoggingConfig()
    if opts["--verbose"]:
        lc.set_verbose(True)
    lc.apply()

    cfg = BookDataConfig.load()

    db_file = Path(opts["DB"])
    if db_file.exists():
        _log.warn("databse %s already exists, removing", db_file)
        db_file.unlink()
    _log.info("opening database %s", db_file)
    with duckdb.connect(db_file) as db:
        register_udfs(db)
        load_graph(db, cfg)
        compute_clusters(db)


def load_graph(db: duckdb.DuckDBPyConnection, cfg: BookDataConfig):
    db.execute(
        """
        CREATE TYPE node_type AS ENUM (
            'ISBN', 'LOC', 'OL-W', 'OL-E', 'GR-B', 'GR-W'
        )
        """
    )
    db.execute(
        """
        CREATE TABLE bg_nodes (
            code INTEGER NOT NULL,
            cluster INTEGER NULL,
            type node_type NOT NULL,
            label VARCHAR NULL,
        )
        """
    )
    db.execute(
        """
        CREATE TABLE bg_edges (
            src INTEGER NOT NULL,
            dst INTEGER NOT NULL,
        )
        """
    )

    load_isbn_nodes(db)
    load_loc_nodes(db)
    load_ol_edition_nodes(db)
    load_ol_work_nodes(db)
    if cfg.goodreads.enabled:
        load_gr_book_nodes(db)
        load_gr_work_nodes(db)

    load_loc_edges(db)
    load_ol_edition_edges(db)
    load_ol_work_edges(db)
    if cfg.goodreads.enabled:
        load_gr_book_edges(db)
        load_gr_work_edges(db)

    _log.info("setting node primary key")
    db.execute("ALTER TABLE bg_nodes ADD PRIMARY KEY (code)")

    _log.info("indexing edge table")
    _log.debug("creating edge src index")
    db.execute("CREATE INDEX edge_src_idx ON bg_edges (src)")
    _log.debug("creating edge dst index")
    db.execute("CREATE INDEX edge_dst_idx ON bg_edges (dst)")


def load_isbn_nodes(db):
    _log.info("loading ISBN nodes")
    db.execute(
        """
        INSERT INTO bg_nodes (code, type, label)
        SELECT isbn_code(isbn_id), 'ISBN', isbn
        FROM 'book-links/all-isbns.parquet'
        """
    )
    (n,) = db.fetchone() or [0]
    _log.info("added %d ISBN nodes", n)


def load_loc_nodes(db):
    _log.info("loading LOC nodes")
    db.execute(
        """
        INSERT INTO bg_nodes (code, type)
        SELECT loc_rec_code(rec_id::INT32), 'LOC',
        FROM 'loc-mds/book-ids.parquet'
        """
    )
    (n,) = db.fetchone() or [0]
    _log.info("added %d LOC nodes", n)


def load_loc_edges(db):
    _log.info("loading LOC edges")
    db.execute(
        """
        INSERT INTO bg_edges (src, dst)
        SELECT isbn_code(isbn_id::INT32), loc_rec_code(rec_id::INT32)
        FROM 'loc-mds/book-isbn-ids.parquet'
        """
    )
    (n,) = db.fetchone() or [0]
    _log.info("added %d LOC edges", n)


def load_ol_edition_nodes(db):
    _log.info("loading OL Edition nodes")
    db.execute(
        """
        INSERT INTO bg_nodes (code, type, label)
        SELECT ol_edition_code(id::INT32), 'OL-E', key
        FROM 'openlibrary/editions.parquet'
        """
    )
    (n,) = db.fetchone() or [0]
    _log.info("added %d OL Edition nodes", n)


def load_ol_edition_edges(db):
    _log.info("loading OL Edition edges")
    db.execute(
        """
        INSERT INTO bg_edges (src, dst)
        SELECT isbn_code(isbn_id::INT32), ol_edition_code(edition::INT32)
        FROM 'openlibrary/edition-isbn-ids.parquet'
        """
    )
    (n,) = db.fetchone() or [0]
    _log.info("added %d OL Edition edges", n)


def load_ol_work_nodes(db):
    _log.info("loading OL Work nodes")
    db.execute(
        """
        INSERT INTO bg_nodes (code, type, label)
        SELECT ol_work_code(id::INT32), 'OL-W', key
        FROM 'openlibrary/works.parquet'
        """
    )
    (n,) = db.fetchone() or [0]
    _log.info("added %d OL Work nodes", n)

    _log.info("adding dangling OL Work nodes")
    db.execute(
        """
        INSERT INTO bg_nodes (code, type)
        SELECT DISTINCT ol_work_code(work::INT32), 'OL-W'
        FROM 'openlibrary/edition-works.parquet'
        WHERE work NOT IN (
            SELECT ol_work_id(code)
            FROM bg_nodes
            WHERE type = 'OL-W'
        )
        """
    )
    (n,) = db.fetchone() or [0]
    _log.info("added %d dangling OL Work nodes", n)


def load_ol_work_edges(db):
    _log.info("loading OL Work edges")
    db.execute(
        """
        INSERT INTO bg_edges (src, dst)
        SELECT  ol_edition_code(edition::INT32), ol_work_code(work::INT32)
        FROM 'openlibrary/edition-works.parquet'
        """
    )
    (n,) = db.fetchone() or [0]
    _log.info("added %d OL Work edges", n)


def load_gr_book_nodes(db):
    _log.info("loading GR Book nodes")
    db.execute(
        """
        INSERT INTO bg_nodes (code, type)
        SELECT gr_book_code(book_id), 'GR-B',
        FROM 'goodreads/gr-book-ids.parquet'
        """
    )
    (n,) = db.fetchone() or [0]
    _log.info("added %d GR Book nodes", n)


def load_gr_book_edges(db):
    _log.info("loading GR Book edges")
    db.execute(
        """
        INSERT INTO bg_edges (src, dst)
        SELECT  isbn_code(isbn_id::INT32), gr_book_code(book_id)
        FROM 'goodreads/book-isbn-ids.parquet'
        """
    )
    (n,) = db.fetchone() or [0]
    _log.info("added %d GR Work edges", n)


def load_gr_work_nodes(db):
    _log.info("loading GR Work nodes")
    db.execute(
        """
        INSERT INTO bg_nodes (code, type)
        SELECT DISTINCT gr_work_code(work_id), 'GR-W',
        FROM 'goodreads/gr-book-ids.parquet'
        WHERE work_id IS NOT NULL
        """
    )
    (n,) = db.fetchone() or [0]
    _log.info("added %d GR Work nodes", n)


def load_gr_work_edges(db):
    _log.info("loading GR Work edges")
    db.execute(
        """
        INSERT INTO bg_edges (src, dst)
        SELECT  gr_book_code(book_id), gr_work_code(work_id)
        FROM 'goodreads/gr-book-ids.parquet'
        WHERE work_id IS NOT NULL
        """
    )
    (n,) = db.fetchone() or [0]
    _log.info("added %d GR Work edges", n)


def compute_clusters(db: duckdb.DuckDBPyConnection):
    db.execute(
        """
        CREATE TEMPORARY TABLE cluster_work (
            code INTEGER PRIMARY KEY,
            cluster INTEGER NOT NULL,
            size INTEGER NOT NULL,
        );
        """
    )
    _log.info("preparing cluster operation")
    db.execute(
        """
        INSERT INTO cluster_work (code, cluster, size)
        SELECT code, code, 1
        FROM bg_nodes
        """
    )
    epoch = 0
    db.execute("SELECT COUNT(code) FROM cluster_work")
    (total,) = db.fetchone() or [0]
    log = _log.bind(nodes=total)
    remaining = total
    while remaining > 0:
        epoch += 1
        elog = log.bind(epoch=epoch, remaining=remaining)
        elog.info("beginning cluster iteration")
        elog.debug("updating cluster IDs")
        db.execute(
            """
            UPDATE cluster_work
            SET cluster = LEAST(cluster_work.cluster, nh.cluster)
            FROM (
                SELECT code, MIN(cluster) AS cluster
                FROM (
                    -- get right edges
                    SELECT src AS code, cluster
                    FROM bg_edges e
                    JOIN cluster_work w ON (w.code = e.dst)
                    -- add left edges
                    UNION ALL
                    SELECT dst AS code, cluster
                    FROM bg_edges e
                    JOIN cluster_work w ON (w.code = e.src)
                ) nbr
                GROUP BY code
            ) nh
            WHERE cluster_work.code = nh.code
            """
        )
        elog.debug("counting cluster sizes")
        db.execute(
            """
            CREATE OR REPLACE TEMPORARY TABLE cluster_sizes AS
            SELECT cluster, COUNT(code) AS size
            FROM cluster_work
            GROUP BY cluster
            """
        )
        elog.debug("saving finished cluster clusters")
        db.execute(
            """
            UPDATE bg_nodes
            SET cluster = w.cluster
            FROM cluster_work w
            JOIN cluster_sizes s USING (cluster)
            WHERE w.code = bg_nodes.code
            AND s.size = w.size
            """
        )
        (n,) = db.fetchone() or []
        elog.info("finalized %d node clusters", n)
        elog.debug("clearing finished clusters from work")
        db.execute(
            """
            DELETE FROM cluster_work AS w
            USING cluster_sizes s
            WHERE w.cluster = s.cluster AND w.size = s.size
            """
        )
        elog.debug("updating unfinished cluster sizes")
        db.execute(
            """
            UPDATE cluster_work
            SET size = cs.size
            FROM cluster_sizes cs
            WHERE cluster_work.cluster = cs.cluster
            """
        )
        elog.debug("counting remaining work nodes")
        db.execute("SELECT COUNT(*) FROM cluster_work")
        (remaining,) = db.fetchone() or [0]

    _log.info("finished clustering after %d epochs", epoch)
    _log.info("committing cluster results")
    db.execute(
        """
        UPDATE bg_nodes
        SET cluster = cw.cluster
        FROM cluster_work cw
        WHERE bg_nodes.code = cw.code
        """
    )
    db.execute("SELECT COUNT(DISTINCT cluster) FROM bg_nodes")
    (count,) = db.fetchone() or []
    _log.info("found %d clusters", count)


if __name__ == "__main__":
    main()
