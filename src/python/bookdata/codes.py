from __future__ import annotations

from dataclasses import dataclass
from functools import partial

import duckdb
import pyarrow as pa
import pyarrow.compute as pc
from duckdb.typing import INTEGER

NS_MULT_BASE = 100_000_000


@dataclass
class NS:
    name: str
    fn_name: str
    code: int

    @property
    def base(self) -> int:
        return self.code * NS_MULT_BASE

    def to_code(self, n: int) -> int:
        if n < 0 or n >= NS_MULT_BASE:
            raise ValueError(f"code {n} out-of-bounds")
        return n * self.base

    def from_code(self, n: int) -> int | None:
        low = self.base
        high = low + NS_MULT_BASE
        if n >= low and n < high:
            return n - low
        else:
            return None

    @staticmethod
    def by_name(name: str) -> NS:
        return _NS_INDEX[name]


NS_WORK = NS("OL-W", "ol_work", 1)
NS_EDITION = NS("OL-E", "ol_edition", 2)
NS_LOC_REC = NS("LOC", "loc_rec", 3)
NS_GR_WORK = NS("GR-W", "gr_work", 4)
NS_GR_BOOK = NS("GR-B", "gr_book", 5)
NS_LOC_WORK = NS("LOC-W", "loc_work", 6)
NS_LOC_INSTANCE = NS("LOC-I", "loc_instance", 7)
NS_ISBN = NS("ISBN", "isbn", 9)

NUMSPACES = [
    NS_WORK,
    NS_EDITION,
    NS_LOC_REC,
    NS_GR_WORK,
    NS_GR_BOOK,
    NS_LOC_WORK,
    NS_LOC_INSTANCE,
    NS_ISBN,
]

_NS_INDEX = {ns.name: ns for ns in NUMSPACES}


def register_udfs(db: duckdb.DuckDBPyConnection):
    for ns in NUMSPACES:
        db.create_function(
            f"{ns.fn_name}_code",
            partial(book_code_udf, ns),
            [INTEGER],
            return_type=INTEGER,
            type="arrow",
        )

        db.create_function(
            f"{ns.fn_name}_id",
            partial(book_id_udf, ns),
            [INTEGER],
            return_type=INTEGER,
            type="arrow",
            null_handling="special",
        )

        db.create_function(
            f"is_{ns.fn_name}",
            partial(book_test_udf, ns),
            [INTEGER],
            return_type=INTEGER,
            type="arrow",
        )


def book_code_udf(ns: NS, ids: pa.Array) -> pa.Array:
    """
    SQL UDF to compute a book code from an ID.
    """
    return pc.add(ids, ns.base)


def book_id_udf(ns: NS, ids: pa.Array) -> pa.Array:
    """
    SQL UDF to compute a book code from an ID.
    """
    sub = pc.subtract(ids, ns.base)
    return pc.if_else(pc.greater_equal(sub, 0), sub, None)


def book_test_udf(ns: NS, ids: pa.Array) -> pa.Array:
    """
    SQL UDF to compute a book code from an ID.
    """
    sub = pc.subtract(ids, ns.base)
    ns = pc.subtract(ids, ns.base + NS_MULT_BASE)
    return pc.and_(pc.greater_equal(sub, 0), pc.less(ns, 0))
