//! Graph extraction program.
use std::convert::From;
use std::path::PathBuf;

use crate::graph::{load_graph, save_gml};
use crate::prelude::*;

/// Extract a subgraph.
#[derive(Args, Debug)]
#[command(name = "extract-graph")]
pub struct ExtractGraph {
    #[arg(long = "graph-file")]
    graph_file: Option<PathBuf>,

    #[arg(short = 'c', long = "cluster")]
    cluster: Option<i32>,

    #[arg(long = "output", short = 'o')]
    out_file: Option<PathBuf>,
}

impl Command for ExtractGraph {
    fn exec(&self) -> Result<()> {
        let path = match &self.graph_file {
            Some(p) => p.clone(),
            None => PathBuf::from("book-links/book-graph.mp.zst"),
        };

        info!("loading graph from {}", path.to_string_lossy());
        let mut graph = load_graph(path)?;

        if let Some(c) = &self.cluster {
            info!("filtering graph for cluster {}", c);
            graph.retain_nodes(|g, n| {
                let book = g.node_weight(n).unwrap();
                book.cluster == *c
            });
        }

        info!("filtered graph to {} nodes", graph.node_count());

        if let Some(outf) = &self.out_file {
            save_gml(&graph, outf)?;
        }

        Ok(())
    }
}
