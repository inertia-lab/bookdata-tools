"""
Export Parquet schema and statistics.

Usage:
    pqinfo [-v] [-o OUT] PARQUET

Options:
    -v, --verbose
        Enable verbose logging output.
    -o OUT, --output=OUT
        Write output to OUT.
    PARQUET
        The Parquet file to summarize.
"""

import json
from logging import getLogger
from pathlib import Path

import pyarrow.parquet as pq
import rich.console
from docopt import docopt
from humanize import metric, naturalsize

from bookdata.logging import setup_logging

_log = getLogger("__name__")


def main():
    opts = docopt(__doc__ or "")
    console = rich.console.Console()

    setup_logging(opts["--verbose"])

    src = Path(opts["PARQUET"])
    fmeta = src.stat()
    _log.info("reading %s (%s)", src, naturalsize(fmeta.st_size))
    meta = pq.read_metadata(src)
    _log.info("row count: %s", metric(meta.num_rows))
    _log.info("row groups: %s", metric(meta.num_rows))

    schema = meta.schema.to_arrow_schema()

    console.print("[bold]Schema:[/bold]")
    for name in schema.names:
        f = schema.field(name)
        console.print(f"[cyan]{name}[/cyan]: {f.type}")

    out_f = opts["--output"]
    if out_f:
        with open(out_f, "wt") as jsf:
            data = json.dumps(
                {
                    "row_count": meta.num_rows,
                    "file_size": fmeta.st_size,
                    "fields": [
                        {"name": name, "data_type": str(schema.field(name).type)}
                        for name in schema.names
                    ],
                },
                indent=2,
            )
            print(data, file=jsf)
