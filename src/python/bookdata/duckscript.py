#!/usr/bin/env python
"""
Run one or more DuckDB scripts (with options) to produce a database file.

Usage:
    duckscript [-v] [-D VSPEC]... [--remove] [--read-only] [-d DB] SCRIPT...

Options:
    -v, --verbose
        Enable verbose log output.
    -D VSPEC, --define=VSPEC
        Define a variable according to VSPEC (NAME=VALUE).
    --remove
        Remove the database file if it exists prior to run.
    --read-only
        Connect to the database read-only.
    -d DB, --database=DB
        The path to the DuckDB database to operate on.
    SCRIPT
        Path to a database script to run on DB.
"""

from __future__ import annotations

import re
from pathlib import Path

from docopt import docopt
from duckdb import DuckDBPyConnection, Statement, StatementType, connect, extract_statements
from lenskit.logging import LoggingConfig, get_logger

from bookdata.codes import register_udfs

logger = get_logger("poprox_recommender.data.duckscript")


def main():
    options = docopt(__doc__ or "")
    log_cfg = LoggingConfig()
    if options["--verbose"]:
        log_cfg.set_verbose(True)
    log_cfg.apply()

    db_path = options["--database"]
    if db_path:
        db_path = Path(db_path)
    if db_path and db_path.exists() and options["--remove"]:
        logger.info("removing database %s", db_path)
        db_path.unlink()

    logger.info("opening database %s", db_path)
    with connect(db_path or ":memory:", read_only=bool(options["--read-only"])) as db:
        register_udfs(db)
        for var in options["--define"]:
            name, value = var.split("=", 1)
            logger.debug("setting %s=%s", name, value)
            db.execute(f"SET VARIABLE {name} = ?", [value])

        for script in options["SCRIPT"]:
            script = Path(script)
            _run_script(db, script)


def _run_script(db: DuckDBPyConnection, script: Path):
    """
    Run an individual script file on the database connection.
    """
    sql = script.read_text()
    statements = extract_statements(sql)
    logger.info("running %d statements from %s", len(statements), script)

    for stmt in statements:
        logger.debug("executing %s statement", stmt.type, query=stmt.query.strip())
        if stmt.type == StatementType.SELECT_STATEMENT:
            # SELECT statements are just to print intermediate results
            logger.info("executed SELECT")
            print(db.query(stmt.query))
        else:
            # all other statement types we execute.
            db.execute(stmt)
            res = db.fetchone()
            meta = {}
            if res is not None:
                (n,) = res
                meta["rows"] = n
            logger.info("executed %s", _pretty_query(stmt), type=str(stmt.type), **meta)


def _pretty_query(stmt: Statement) -> str:
    # grab the statement words up to the first non-capitalized variable.
    # i.e., CREATE TABLE foo (...) becomes CREATE TABLE foo
    query = re.sub("--.*$", "", stmt.query, flags=re.MULTILINE)
    m = re.match(r".*?\b((?:[A-Z]+ )+[a-z0-9_]+)\b", query, flags=re.DOTALL)
    if m:
        pretty = m.group(1)
    else:
        logger.warn("unparsable query: %s", query)
        pretty = query
    pretty = re.sub(r"\s+", " ", pretty)
    return pretty


if __name__ == "__main__":
    main()
