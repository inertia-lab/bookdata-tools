"""
Create the book graph.

Usage:
    make-graph.py [-v] DB

Args:
    -v, --verbose
        Enable verbose log output.
    DB
        The database file to create.
"""

import json

import duckdb
import igraph
import numpy as np
import pandas as pd
from bookdata.codes import register_udfs
from bookdata.config import BookDataConfig
from docopt import docopt
from humanize import metric
from lenskit.logging import LoggingConfig, get_logger

_log = get_logger("cluster")


def main():
    opts = docopt(__doc__ or "")
    lc = LoggingConfig()
    if opts["--verbose"]:
        lc.set_verbose(True)
    lc.apply()

    cfg = BookDataConfig.load()

    db_path = opts["DB"]
    _log.info("opening database %s", db_path)
    with duckdb.connect(db_path) as db:
        register_udfs(db)
        load_graph(db, cfg)
        compute_clusters(db)
        cluster_metrics(db)


def load_graph(db: duckdb.DuckDBPyConnection, cfg: BookDataConfig):
    db.execute(
        """
        CREATE TYPE node_type AS ENUM (
            'ISBN', 'LOC', 'OL-W', 'OL-E', 'GR-B', 'GR-W'
        )
        """
    )
    db.execute("CREATE SEQUENCE node_id_seq START WITH 1")
    db.execute(
        """
        CREATE TABLE bg_nodes (
            node_id INTEGER NOT NULL DEFAULT nextval('node_id_seq'),
            code INTEGER NOT NULL,
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
    db.execute(
        """
        CREATE VIEW bg_neighbors AS
        SELECT src AS node, dst AS nbr
        FROM bg_edges
        UNION ALL
        SELECT dst AS node, src AS nbr
        FROM bg_edges
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

    # _log.info("setting node primary key")
    # db.execute("ALTER TABLE bg_nodes ADD PRIMARY KEY (code)")

    # _log.info("indexing edge table")
    # _log.debug("creating edge src index")
    # db.execute("CREATE INDEX edge_src_idx ON bg_edges (src)")
    # _log.debug("creating edge dst index")
    # db.execute("CREATE INDEX edge_dst_idx ON bg_edges (dst)")


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
    _log.info("added %d GR Book edges", n)


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
    _log.info("loading graph nodes")
    db.execute(
        """
        SELECT code
        FROM bg_nodes
        ORDER BY node_id
        """
    )
    nodes = db.fetch_df()

    _log.info("loading graph edges")
    db.execute(
        """
        SELECT sn.node_id - 1 AS src_id, dn.node_id - 1 AS dst_id
        FROM bg_edges e
        JOIN bg_nodes sn ON e.src = sn.code
        JOIN bg_nodes dn ON e.dst = dn.code
        """
    )
    edges = db.fetch_df()

    _log.info("assembling graph")
    graph = igraph.Graph.DataFrame(edges, False, nodes)
    _log.info(
        "loaded graph with %s nodes and %s edges", metric(len(graph.vs)), metric(len(graph.es))
    )

    _log.info("computing connected components")
    cc = graph.connected_components()
    clusters = pd.DataFrame(
        {"node_id": np.arange(1, len(graph.vs) + 1, dtype=np.int32), "cluster": cc.membership}
    )
    _log.info("computed %s clusters", metric(clusters["cluster"].nunique()))
    db.execute(
        """
        CREATE TABLE node_clusters AS
        SELECT code, cluster
        FROM clusters
        JOIN bg_nodes USING (node_id)
        ORDER BY code
        """
    )


def cluster_metrics(db: duckdb.DuckDBPyConnection):
    db.execute(
        """
        CREATE VIEW cluster_stats AS
        SELECT cluster,
            COUNT(code) AS n_nodes,
            COUNT(code) FILTER (type = 'ISBN') AS n_isbns,
            COUNT(code) FILTER (type = 'LOC') AS n_loc_recs,
            COUNT(code) FILTER (type = 'OL-E') AS n_ol_editions,
            COUNT(code) FILTER (type = 'OL-W') AS n_ol_works,
            COUNT(code) FILTER (type = 'GR-B') AS n_gr_editions,
            COUNT(code) FILTER (type = 'GR-W') AS n_gr_works,
        FROM bg_nodes
        JOIN node_clusters USING (code)
        GROUP BY cluster
        """
    )
    db.execute(
        """
        SELECT SUM(n_nodes), COUNT(*), MAX(n_nodes), MAX(n_isbns)
        FROM cluster_stats
        """
    )
    nodes, clusters, largest, max_isbns = db.fetchone() or []
    with open("book-links/cluster-metrics.json", "wt") as jsf:
        print(
            json.dumps(
                {
                    "nodes": nodes,
                    "clusters": clusters,
                    "largest": largest,
                    "max_isbns": max_isbns,
                }
            ),
            file=jsf,
        )


if __name__ == "__main__":
    main()
