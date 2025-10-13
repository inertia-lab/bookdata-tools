local bd = import '../bookdata.libsonnet';
local source_stages = {
  ratings: {
    'scan-ratings': {
      cmd: bd.cmd('amazon scan-ratings -o ratings.parquet --swap-id-columns ../data/az2018/Books.csv'),
      deps: [
        '../src/amazon.rs',
        '../src/cli/amazon/',
        '../data/az2018/Books.csv',
      ],
      outs: ['ratings.parquet'],
    },
  },
  reviews: {
    'scan-reviews': {
      cmd: bd.cmd('amazon scan-reviews --rating-output ratings.parquet --review-output reviews.parquet ../data/az2018/Books.json.gz'),
      deps: [
        '../src/amazon.rs',
        '../src/cli/amazon/',
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
    wdir: '..',
    cmd: 'duckscript az-cluster-ratings.sql',
    deps: [
      'src/cli/amazon',
      'az2018/ratings.parquet',
      'book-links/isbn-clusters.parquet',
    ],
    outs: ['az2018/az-cluster-ratings.parquet'],
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
