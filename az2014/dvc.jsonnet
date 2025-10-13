local bd = import '../bookdata.libsonnet';

bd.pipeline({
  'scan-ratings': {
    cmd: 'duckscript az-scan-ratings.sql',
    deps: [
      'az-scan-ratings.sql',
      '../data/az2014/ratings_Books.csv',
    ],
    outs: ['ratings.parquet'],
  },

  'cluster-ratings': {
    wdir: '..',
    cmd: 'duckscript az-cluster-ratings.sql',
    deps: [
      'src/cli/amazon',
      'az2014/ratings.parquet',
      'book-links/isbn-clusters.parquet',
    ],
    outs: ['az2014/az-cluster-ratings.parquet'],
  },

  'cluster-ratings-5core': {
    cmd: 'python ../scripts/kcore.py -o az-cluster-ratings-5core.parquet az-cluster-ratings.parquet',
    deps: [
      'az-cluster-ratings.parquet',
      '../scripts/kcore.py',
    ],
    outs: ['az-cluster-ratings-5core.parquet'],
  },
}, bd.config.az2014.enabled)
