local bd = import '../bookdata.libsonnet';
local source_stages = {
  ratings: {
    'scan-ratings': {
      cmd: 'duckscript az-scan-ratings.sql',
      deps: [
        'az-scan-ratings.sql',
        '../data/az2018/Books.csv',
      ],
      outs: ['ratings.parquet'],
    },
  },
  reviews: {
    'scan-reviews': {
      cmd: 'duckscript az-scan-reviews.sql',
      deps: [
        'az-scan-reviews.sql',
        '../data/az2018/Books.json.gz',
      ],
      outs: [
        'ratings.parquet',
        'reviews.parquet',
      ],
    },
  },
};

bd.pipeline(source_stages[bd.config.az2018.source] {
  'cluster-ratings': {
    cmd: 'duckscript az-cluster-ratings.sql',
    deps: [
      'az-cluster-ratings.sql',
      'ratings.parquet',
      '../book-links/isbn-clusters.parquet',
    ],
    outs: ['az-cluster-ratings.parquet'],
  },

  'cluster-ratings-5core': {
    cmd: 'python ../scripts/kcore.py -o az-cluster-ratings-5core.parquet az-cluster-ratings.parquet',
    deps: [
      'az-cluster-ratings.parquet',
      '../scripts/kcore.py',
    ],
    outs: ['az-cluster-ratings-5core.parquet'],
  },
}, bd.config.az2018.enabled)
