//! Code for processing and integrating book data.
//!
//! The book data tools are developed as a monolithic executable.  The commands
//! themselves live under [cli], while the rest of the package contains data
//! definitions and helper routines that build on this code.  The tools are not
//! currently usable as a library; you can extend them by adding additional commands
//! to the [cli] module (`src/cli/` in the source tree).
mod arrow;
mod cleaning;
mod cli;
mod gender;
mod goodreads;
mod ids;
mod interactions;
mod io;
mod layout;
mod marc;
mod openlib;
mod parsing;
mod prelude;
mod tsv;
mod util;

use mimalloc::MiMalloc;
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

// use snmalloc_rs::SnMalloc;
// #[global_allocator]
// static GLOBAL: SnMalloc = SnMalloc;

use anyhow::Result;
use clap::Parser;

use cli::CLI;

fn main() -> Result<()> {
    let opt = CLI::parse();
    opt.exec()
}
