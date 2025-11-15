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

from logging import getLogger

import duckdb
import pyprojroot
from bookdata.logging import setup_logging
from docopt import ParsedOptions, docopt

_log = getLogger("link-isbns")

ROOT = pyprojroot.find_root(pyprojroot.has_dir(".git"))


def main(opts: ParsedOptions):
    setup_logging()
    out = opts["--output"]
    rec_field = opts["--record-id"]
    isbn_field = opts["--isbn-field"]
    src = opts["INPUT"]

    with duckdb.connect() as db:
        isbn_db = ROOT / "book-links" / "isbns.db"
        _log.info("attaching ISBN database %s", isbn_db)
        db.execute(f"ATTACH '{isbn_db}' AS isbns (READONLY)")

        if len(isbn_field) == 1:
            _log.info("making single-ISBN query for field %s", isbn_field)
            query = f"""
                SELECT DISTINCT {rec_field}, isbn_id
                FROM '{src}' s
                JOIN isbns.isbns i ON (s.{isbn_field[0]} = i.isbn)
                ORDER BY {rec_field}
            """
        else:
            _log.info("making multi-ISBN query")
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

        _log.info("generating ISBN link file")
        db.execute(f"COPY ({query}) TO '{out}' (COMPRESSION zstd)")


if __name__ == "__main__":
    opts = docopt(__doc__ or "")
    main(opts)
