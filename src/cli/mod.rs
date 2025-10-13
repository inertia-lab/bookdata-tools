//! Command line interface for book data.
//!
//! The book data tools are implemented as a single monolithic executable (to reduce
//! compilation time and disk space in common configurations, with different tools
//! implemented as subcommands.  Each subcommand implements the [Command] trait, which
//! exposes the command line arguments and invocation.
pub mod amazon;
pub mod bx;
pub mod cluster;
pub mod cluster_books;
pub mod extract_graph;
pub mod filter_marc;
pub mod goodreads;
pub mod index_names;
pub mod openlib;
pub mod pqinfo;
pub mod scan_marc;
pub mod stats;

use anyhow::Result;
use clap::{Parser, Subcommand};
use cpu_time::ProcessTime;
use enum_dispatch::enum_dispatch;
use happylog::clap::LogOpts;
use log::*;
use paste::paste;

use crate::util::process;
use crate::util::Timer;

/// Macro to generate wrappers for subcommand enums.
///
/// This is for subcommands that only exist to contain further subcommands,
/// to make it easier to implement their wrapper classes.
macro_rules! wrap_subcommands {
    ($name:ty) => {
        paste! {
          #[derive(clap::Args, Debug)]
          pub struct [<$name Wrapper>] {
            #[command(subcommand)]
            command: $name
          }

          impl Command for [<$name Wrapper>] {
            fn exec(&self) -> Result<()> {
              self.command.exec()
            }
          }
        }
    };
}

/// Trait implemented by book data commands.
#[enum_dispatch]
pub trait Command {
    /// Run the command with options
    fn exec(&self) -> Result<()>;
}

/// Enum to collect and dispatch CLI commands.
#[enum_dispatch(Command)]
#[derive(Subcommand, Debug)]
pub enum RootCommand {
    ScanMARC(scan_marc::ScanMARC),
    FilterMARC(filter_marc::FilterMARC),
    ClusterBooks(cluster_books::ClusterBooks),
    IndexNames(index_names::IndexNames),
    ExtractGraph(extract_graph::ExtractGraph),
    /// Commands for processing Amazon data.
    Amazon(AmazonCommandWrapper),
    /// Commands for processing OpenLibrary data.
    Openlib(openlib::OpenLib),
    /// Commands for processing BookCrossing data.
    BX(BXCommandWrapper),
    /// Commands for processing GoodReads data.
    Goodreads(goodreads::Goodreads),
    /// Commands for working with clusters.
    Cluster(ClusterCommandWrapper),
    PQInfo(pqinfo::PQInfo),
    IntegrationStats(stats::IntegrationStats),
}

wrap_subcommands!(AmazonCommand);
wrap_subcommands!(BXCommand);
wrap_subcommands!(ClusterCommand);

#[enum_dispatch(Command)]
#[derive(Subcommand, Debug)]
enum AmazonCommand {
    ScanRatings(amazon::ScanRatings),
    ScanReviews(amazon::ScanReviews),
    ClusterRatings(amazon::ClusterRatings),
}

#[enum_dispatch(Command)]
#[derive(Subcommand, Debug)]
enum BXCommand {
    /// Extract BX from source data and clean.
    Extract(bx::Extract),
    /// Match BX interactions with clusters.
    ClusterActions(bx::Cluster),
}

#[enum_dispatch(Command)]
#[derive(Subcommand, Debug)]
pub enum ClusterCommand {
    Hash(cluster::hash::HashCmd),
    ExtractBooks(cluster::books::ExtractBooks),
    ExtractAuthors(cluster::authors::ClusterAuthors),
    ExtractAuthorGender(cluster::author_gender::AuthorGender),
}

/// Entry point for the Book Data Tools.
///
/// This program runs the various book data tools, exposed as subcommands.
#[derive(Parser, Debug)]
#[command(name = "bookdata")]
pub struct CLI {
    #[command(flatten)]
    logging: LogOpts,

    #[command(subcommand)]
    command: RootCommand,
}

impl CLI {
    pub fn exec(self) -> Result<()> {
        self.logging.init()?;

        process::maybe_exit_early()?;

        let timer = Timer::new();

        let res = self.command.exec();
        if let Err(e) = &res {
            error!("command failed: {}", e);
            return res;
        }

        info!("work completed in {}", timer.human_elapsed());
        match ProcessTime::try_now() {
            Ok(pt) => info!("used {} CPU time", friendly::duration(pt.as_duration())),
            Err(e) => error!("error fetching CPU time: {}", e),
        };

        process::log_process_stats();
        res
    }
}
