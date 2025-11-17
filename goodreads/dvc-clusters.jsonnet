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
    cmd: 'duckscript gr-cluster-ratings.sql',
    deps: [
      'gr-cluster-ratings.sql',
      'gr-book-link.parquet',
      'gr-interactions.parquet',
    ],
    outs: [
      'gr-cluster-ratings.parquet',
    ],
  },

  'cluster-ratings-5core': {
    cmd: 'python ../scripts/kcore.py -o gr-cluster-ratings-5core.parquet gr-cluster-ratings.parquet',
    deps: [
      '../scripts/kcore.py',
      'gr-cluster-ratings.parquet',
    ],
    outs: [
      'gr-cluster-ratings-5core.parquet',
    ],
  },

  'cluster-actions-5core': {
    cmd: 'python ../scripts/kcore.py -o gr-cluster-actions-5core.parquet gr-cluster-actions.parquet',
    deps: [
      '../scripts/kcore.py',
      'gr-cluster-actions.parquet',
    ],
    outs: [
      'gr-cluster-actions-5core.parquet',
    ],
  },
}
