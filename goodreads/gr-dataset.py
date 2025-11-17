"""
Create a LensKit dataset for GoodReads recommendation data.

Usage:
    gr-dataset.py [-v] --works [-a ACTIONS] DIR

Options:
    -v, --verbose
            enable verbose log output
    --works
            create a summarized work-level dataset
    -a ACTIONS, --actions=ACTIONS
            use actions from ACTIONS [default: gr-work-actions.parquet]
    DIR     write dataset to DIR
"""

import sys
from logging import getLogger
from pathlib import Path

import pandas as pd
from bookdata.logging import setup_logging
from docopt import docopt
from lenskit.data import DatasetBuilder
from scipy.sparse import csr_array

_log = getLogger("gr-dataset")

GR_WORK_BASE = 400_000_000
GR_BOOK_BASE = 500_000_000


def main():
    opts = docopt(__doc__ or "")
    setup_logging(opts["--verbose"], lenskit=True)

    out_dir = Path(opts["DIR"])

    if opts["--works"]:
        data = work_dataset(opts)
    else:
        _log.error("no mode specified")
        sys.exit(2)

    _log.info("saving dataset")
    data.save(out_dir)


def work_dataset(opts) -> DatasetBuilder:
    log = _log.bind(items="works")
    bld = DatasetBuilder("gr-works")

    act_f = opts["--actions"]
    log.debug("loading actions from %s", act_f)
    actions = pd.read_parquet(act_f)
    log.info("loaded actions", action_count=len(actions), file=act_f)
    actions = actions.rename(
        columns={
            "first_time": "timestamp",
            "last_rating": "rating",
        }
    )
    actions["timestamp"] = pd.to_datetime(actions["timestamp"], unit="s")
    actions["last_time"] = pd.to_datetime(actions["last_time"], unit="s")
    log.info("adding actions to builder")
    bld.add_interactions(
        "shelve", actions, entities=["user", "item"], missing="insert", default=True
    )
    action_items = pd.Index(actions["item_id"].unique())

    log.info("adding genres")
    genres = pd.read_parquet("gr-genres.parquet")
    genres = genres.sort_values("genre_id")
    work_genres = pd.read_parquet("gr-work-item-genres.parquet")
    work_genres = work_genres[work_genres["item_id"].isin(action_items)]
    row_idx = pd.Index(work_genres["item_id"].unique())
    rows = row_idx.get_indexer_for(work_genres["item_id"]).astype("int32")
    genre_mat = csr_array(
        (work_genres["count"].astype("int16"), (rows, work_genres["genre_id"].values - 1))
    )
    bld.add_vector_attribute(
        "item", "genres", row_idx.values, genre_mat.toarray(), dim_names=genres["genre"].tolist()
    )

    log.info("adding titles")
    titles = pd.read_parquet("gr-work-item-titles.parquet")
    titles = titles[["item_id", "title"]].drop_duplicates("item_id")
    titles = titles[titles["item_id"].isin(action_items)]
    titles = titles.set_index("item_id")
    bld.add_scalar_attribute("item", "title", titles)

    return bld


if __name__ == "__main__":
    main()
