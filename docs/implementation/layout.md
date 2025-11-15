# Code Layout

The import code consists primarily of Python and DuckDB SQL, along with Rust for
data ingest tasks that are difficult to implement efficiently with DuckDB.  This
code is orchestrated with DVC, with data in several directories to facilitate
ease of discovery.  We use Python and R in Quarto documents for analytics and
reporting.

## Rust

The Rust code all lives under `src`, with the various command-line programs in
`src/cli`.  The Rust tools are implemented as a monolithic executable with
subcommands for various operations, to save disk space and compile time.  To see
the help:

    cargo run help

The programs are run through `cargo run` in `--release` mode; the
[`bd.cmd`](pipeline.md#bd.cmd) jsonnet function automates this, so we only need
to specify the subcommand and its options in our pipeline definitions.

For writing new commands, there is a lot of utility code under `src`.  Consult
the [Rust API documentation](/apidocs/bookdata/) for further details.

The Rust code makes extensive use of the [arrow][] and [parquet][] crates for
data analysis and IO.

[arrow]: https://docs.rs/arrow
[parquet]: https://docs.rs/parquet
