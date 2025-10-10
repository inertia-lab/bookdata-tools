#!/usr/bin/env python
"""
Limit a user-item file to a k-core.

Usage:
    kcore [options] -o OUT SRC

Options:
    -k K, --k=K
        Size of core [default: 5].
    -U UK, --user-k=UK
        User action count for (ku,ki)-core.
    -I IK, --item-k=IK
        Item action count for (ku,ki)-core.
    --year=YEAR
        Limit to actions in a particular year.
    --start-date=DATE
        Limit to actions after a particular date.
    --end-date=DATE
        Limit to actions before a partiuclar date.
    -o OUT, --output=OUT
        Write output to OUT.
    SRC
        Input file.
"""

import duckdb
from docopt import ParsedOptions, docopt
from lenskit.logging import basic_logging, get_logger

_log = get_logger("kcore")


def main(opts: ParsedOptions):
    basic_logging()

    src = opts["SRC"]
    dst = opts["--output"]
    k = int(opts["--k"])

    # parse separate user/item options
    if ku := opts["--user-k"]:
        ku = int(ku)
    else:
        ku = k
    if ki := opts["--item-k"]:
        ki = int(ki)
    else:
        ki = k

    with duckdb.connect() as db:
        _log.info("loading ratings from %s", src)
        query = f"SELECT * FROM '{src}'"
        limits = []
        year = opts["--year"]
        params = {}

        if start := opts["--start-date"]:
            _log.info("only actions after %s", start)
            params["start"] = start
            limits.append("first_time >= $start")

        if end := opts["--end-date"]:
            _log.info("only actions before %s", start)
            params["end"] = end
            limits.append("first_time < $end")

        if year:
            params["year"] = year
            limits.append("EXTRACT(YEAR FROM first_time) = $year")

        if limits:
            query += " WHERE " + " AND ".join(limits)

        db.execute(f"CREATE TABLE actions AS {query}", params)

        filter_core(db, ku, ki)

        _log.info("saving to %s", dst)
        db.execute(f"COPY actions TO '{dst}' (COMPRESSION zstd)")


def filter_core(db: duckdb.DuckDBPyConnection, ku: int, ki: int):
    db.execute("SELECT COUNT(*) FROM actions")
    db.execute(
        "CREATE VIEW user_counts AS SELECT user_id, count(*) AS n FROM actions GROUP BY user_id"
    )
    db.execute(
        "CREATE VIEW item_counts AS SELECT item_id, count(*) AS n FROM actions GROUP BY item_id"
    )
    n_iters = 0
    removed = -1
    while removed:
        removed = 0
        n_iters += 1

        _log.info("pass %d: filtering items", n_iters)
        db.execute(
            """
            DELETE FROM actions
            WHERE item_id IN (
                SELECT item_id FROM item_counts
                WHERE n < ?
            )
            """,
            [ki],
        )
        (n,) = db.fetchone() or (-1,)
        removed += n
        _log.info("removed %d actions", n)

        _log.info("pass %d: filtering users", n_iters)
        db.execute(
            """
            DELETE FROM actions
            WHERE user_id IN (
                SELECT user_id FROM user_counts
                WHERE n < ?
            )
            """,
            [ku],
        )
        (n,) = db.fetchone() or (-1,)
        removed += n
        _log.info("removed %d actions", n)

    _log.info("finished after %d passes", n_iters)


if __name__ == "__main__":
    opts = docopt(__doc__ or "")
    main(opts)
