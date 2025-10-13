#!/usr/bin/env python
"""
Generate table linking identifiers to ISBNs.

Usage:
    link-isbns.py [-R FIELD] [-I FIELD]... -o OUT INPUT

Options:
    -I FIELD, --isbn-field=FIELD
        Read ISBN from FIELD [default: isbn].
    -R FIELD, --record-id=FIELD
        Read record IDs from FIELD [default: rec_id].
    -o OUT, --output=OUT
        Write output to OUT.
    INPUT
        Read record ISBNs from file INPUT.
"""

import duckdb
import pyprojroot
from docopt import ParsedOptions, docopt
from lenskit.logging import basic_logging, get_logger

_log = get_logger("link-isbns")

ROOT = pyprojroot.find_root(pyprojroot.has_dir(".git"))


def main(opts: ParsedOptions):
    basic_logging()
    out = opts["--output"]
    rec_field = opts["--record-id"]
    isbn_field = opts["--isbn-field"]
    src = opts["INPUT"]
    log = _log.bind(record=rec_field)

    with duckdb.connect() as db:
        log.info("attaching ISBN database")
        isbn_db = ROOT / "book-links" / "isbns.db"
        db.execute(f"ATTACH '{isbn_db}' AS isbns (READONLY)")

        if len(isbn_field) == 1:
            log.info("making single-ISBN query")
            query = f"""
                SELECT DISTINCT {rec_field}, isbn_id
                FROM '{src}' s
                JOIN isbns.isbns i ON (s.{isbn_field[0]} = i.isbn)
                ORDER BY {rec_field}
            """
        else:
            log.info("making multi-ISBN query")
            up_expr = ", ".join(isbn_field)
            query = f"""
                SELECT DISTINCT {rec_field}, isbn_id
                FROM (
                    UNPIVOT '{src}'
                    ON {up_expr}
                    INTO NAME field VALUE isbn
                ) s
                JOIN isbns.isbns i USING (isbn)
                ORDER BY {rec_field}
            """

        log.info("generating ISBN link file")
        db.execute(f"COPY ({query}) TO '{out}' (COMPRESSION zstd)")


if __name__ == "__main__":
    opts = docopt(__doc__ or "")
    main(opts)
