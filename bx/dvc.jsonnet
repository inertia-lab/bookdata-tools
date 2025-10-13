local bd = import '../bookdata.libsonnet';

bd.pipeline({
  'clean-ratings': {
    cmd: bd.cmd('bx extract ../data/BX-CSV-Dump.zip cleaned-ratings.csv'),
    deps: [
      '../src/cli/bx',
      '../data/BX-CSV-Dump.zip',
    ],
    outs: [
      'cleaned-ratings.csv',
    ],
  },
  'cluster-ratings': {
    cmd: 'duckscript bx-cluster-ratings.sql',
    deps: [
      'bx-cluster-ratings.sql',
      'cleaned-ratings.csv',
      '../book-links/isbn-clusters.parquet',
    ],
    outs: [
      'bx-cluster-ratings.parquet',
    ],
  },
  'cluster-actions': {
    cmd: 'duckscript bx-cluster-actions.sql',
    deps: [
      'bx-cluster-actions.sql',
      'cleaned-ratings.csv',
      '../book-links/isbn-clusters.parquet',
    ],
    outs: [
      'bx-cluster-actions.parquet',
    ],
  },
}, bd.config.bx.enabled)
