# History

This page documents the release history of the Book Data Tools. Each numbered,
released version has a corresponding Git tag (e.g. `v3.0`).

If you use the Book Data Tools in published research, we ask that you do the
following:

1.  Cite the [UMUAI paper](https://md.ekstrandom.net/pubs/bag-extended),
    regardless of which version of the data set you use.
2.  Cite the papers corresponding to the individual ratings, review, or
    consumption data sets you are using.
3.  Clearly state the version of the data tools you are using in your paper.
4.  [Let us know](papers.md) about your work so we can add you to the list.

## Book Data 4.0 (in progress)

-   🐞 Fixed a bug in processing GoodReads genre counts into a LensKit dataset
    that allowed some counts to be negative.
-   Further simplification of the integration code, replacing Polars and a lot
    of custom Rust code with DuckDB.  Rust is now primarily used for initial
    data ingest, along with some integration computations that we have not yet
    figured out how to write efficiently in DuckDB SQL.

## Book Data 3.0

-   Make the pipeline configurable so individual rating datasets can be disabled.
-   Switched from base Anaconda to [Pixi][] for easier environment management.
-   Only support the full JSON GoodReads interaction data, because it is now
    publicly available.
-   Use [jsonnet](implementation/pipeline.md) to generate DVC pipelines, taking
    configuration settings into account.
-   Extract GoodReads author information into {{< file goodreads/gr-author-info.parquet >}}.
-   Extract more work-specific GoodReads information.
-   Support full-text reviews from the GoodReads and Amazon 2018 data sets (enabled by default).
-   Disable the [BookCrossing](data/bx.qmd) data by default since the source website is offline.
-   Extract 5-cores of interaction files.
-   Update to newer OpenLibrary and VIAF dumps (OpenLibrary 2023-12-31, VIAF
    2024-08-04).  Users will need to manually specify a current VIAF dump, since
    those are not archived.

[Pixi]: https://pixi.sh

### Bugs Fixed

-   🪲 GoodReads cluster & work rating timestamps were on incorrect scale

## Book Data 2.1

Version 2.1 has a few updates but does not change existing data schemas when run
with the full GoodReads interaction files.  It does have improved book/author
linking that increases coverage due to a revised and corrected name parsing &
normalization flow.

The tools now support the GoodReads interaction CSV file, which is available
without registration, and uses this by default.  See the [GoodReads data
docs](data/goodreads.qmd) for the details.  This means that, in their default
configuration, the book data integration uses only data that is publicly
available without special request.

### Data Updates

-   Updated VIAF to May 1, 2022 dump
-   Updated OpenLibrary to March 29, 2022 dump
-   Added 2018 version of the Amazon ratings
-   Added code to extract edition and work subjects
-   Updated docs for current extraction layout
-   Added {{< file openlibrary/work-clusters.parquet >}} to simplify OpenLibrary integration

### Logic Updates

-   Switched from DataFusion to [Polars](https://www.pola.rs/), to reduce volatility and improve
    maintainability.  This also involved a switch from Arrow to Arrow2, which seems to have cleaner
    code (and less custom logic needed for IO).
-   Rewrote logic that was previously in DataFusion + custom TCL in Rust, so all integration code
    is in Rust for consistency (and to avoid redundancy in things like logging configuration between
    Rust and Python).  The code is now in 2 languages: Rust integration and Python notebooks to report
    on integration statistics.
-   Improved name parsing
    -   Replaced `nom`-based name parser for {{< rust-fn ~bookdata::cleaning::names::name_variants >}}
        with a new one written in [`peg`][peg], that is both easier to read/maintain and more efficient.
    -   Corrected errors in name parser that emitted empty-string names for some authors.
    -   Added `clean_name` function, used across all name formatting, to normalize whitespace and
        punctuation in name records from any source.
    -   Added more tests for name parsing and normalization.
-   Fixed a bug in GoodReads integration, where we were not extracting ASINs.
-   Extract book genres and series from GoodReads.
-   Updated various Rust dependencies, and upgraded from StructOpt to `clap`'s derive macros.
-   Better progress reporting for data scans.

[peg]: https://docs.rs/peg

## Book Data 2.0

This is the updated release of the Book Data Tools, using the same source data
as 1.0 but with DataFusion and Rust-based import logic, instead of PostgreSQL.
It is significantly easier to install and use.

## Book Data 1.0

The original release that used PostgreSQL. There were a couple of versions of
this for the RecSys and UMUAI papers; the tagged 1.0 release corresponds to the
data used for the UMUAI paper.
