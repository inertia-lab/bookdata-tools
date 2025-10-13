use crate::prelude::*;

mod cluster;
mod scan;

/// GoodReads processing commands.
#[derive(Args, Debug)]
pub struct Goodreads {
    #[command(subcommand)]
    command: GRCmd,
}

#[derive(clap::Subcommand, Debug)]
enum GRCmd {
    /// Scan GoodReads data.
    Scan {
        #[command(subcommand)]
        data: scan::GRScan,
    },
    /// Cluster GoodReads intearaction data.
    ClusterInteractions(cluster::CICommand),
}

impl Command for Goodreads {
    fn exec(&self) -> Result<()> {
        match &self.command {
            GRCmd::Scan { data } => {
                data.exec()?;
            }
            GRCmd::ClusterInteractions(opts) => {
                opts.exec()?;
            }
        }

        Ok(())
    }
}
