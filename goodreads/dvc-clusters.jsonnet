local bd = import '../bookdata.libsonnet';

{
  'cluster-actions': {
    cmd: 'duckscript gr-cluster-actions.sql',
    deps: [
      'gr-cluster-actions.sql',
      'gr-book-link.parquet',
      'gr-interactions.parquet',
    ],
    outs: [
      'gr-cluster-actions.parquet',
    ],
  },

  'cluster-ratings': {
    wdir: '..',
    cmd: bd.cmd('goodreads cluster-interactions --ratings -o goodreads/gr-cluster-ratings.parquet'),
    deps: [
      'src/cli/goodreads/cluster.rs',
      'goodreads/gr-book-link.parquet',
      'goodreads/gr-interactions.parquet',
    ],
    outs: [
      'goodreads/gr-cluster-ratings.parquet',
    ],
  },

  'cluster-ratings-5core': {
    cmd: bd.cmd('kcore -o gr-cluster-ratings-5core.parquet gr-cluster-ratings.parquet'),
    deps: [
      '../src/cli/kcore.rs',
      'gr-cluster-ratings.parquet',
    ],
    outs: [
      'gr-cluster-ratings-5core.parquet',
    ],
  },

  'cluster-actions-5core': {
    cmd: bd.cmd('kcore -o gr-cluster-actions-5core.parquet gr-cluster-actions.parquet'),
    deps: [
      '../src/cli/kcore.rs',
      'gr-cluster-actions.parquet',
    ],
    outs: [
      'gr-cluster-actions-5core.parquet',
    ],
  },
}
