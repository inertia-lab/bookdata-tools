local bd = import '../bookdata.libsonnet';

bd.pipeline({
  'collect-isbns': {
    cmd: 'python collect-isbns.py -d isbns.db -o all-isbns.parquet',
    deps: std.prune([
      '../config.yaml',
      'collect-isbns.py',
      '../loc-mds/book-isbns.parquet',
      '../openlibrary/edition-isbns.parquet',
      bd.maybe(bd.config.goodreads.enabled, '../goodreads/gr-book-ids.parquet'),
      bd.maybe(bd.config.bx.enabled, '../bx/cleaned-ratings.csv'),
      bd.maybe(bd.config.az2014.enabled, '../az2014/ratings.parquet'),
      bd.maybe(bd.config.az2018.enabled, '../az2018/ratings.parquet'),
    ]),
    outs: [
      'isbns.db',
      'all-isbns.parquet',
    ],
  },

  'book-graph': {
    wdir: '..',
    cmd: 'python book-links/make-graph.py book-links/book-graph.db',
    deps: [
      'book-links/make-graph.py',
      'book-links/all-isbns.parquet',
      'loc-mds/book-ids.parquet',
      'loc-mds/book-isbn-ids.parquet',
      'openlibrary/editions.parquet',
      'openlibrary/works.parquet',
      'openlibrary/edition-isbn-ids.parquet',
      'openlibrary/edition-works.parquet',
    ] + if bd.config.goodreads.enabled then [
      'goodreads/gr-book-ids.parquet',
      'goodreads/book-isbn-ids.parquet',
    ] else [],
    outs: [
      'book-links/book-graph.db',
    ],
    metrics: [{
      'book-links/cluster-metrics.json': {
        cache: false,
      },
    }],
  },

  'export-clusters': {
    cmd: 'duckscript --read-only -d book-graph.db export-clusters.sql',
    deps: [
      'export-clusters.sql',
      'book-graph.db',
    ],
    outs: [
      'book-graph-nodes.parquet',
      'book-graph-edges.parquet',
      'isbn-clusters.parquet',
      'cluster-stats.parquet',
    ],
  },

  'cluster-ol-first-authors': {
    cmd: 'duckscript cluster-ol-first-authors.sql',
    deps: [
      'cluster-ol-first-authors.sql',
      'isbn-clusters.parquet',
      '../openlibrary/edition-isbn-ids.parquet',
      '../openlibrary/edition-authors.parquet',
      '../openlibrary/author-names.parquet',
    ],
    outs: [
      '../book-links/cluster-ol-first-authors.parquet',
    ],
  },

  'cluster-loc-first-authors': {
    cmd: 'duckscript cluster-loc-first-authors.sql',
    deps: [
      'cluster-loc-first-authors.sql',
      'isbn-clusters.parquet',
      '../loc-mds/book-isbn-ids.parquet',
      '../loc-mds/book-authors.parquet',
    ],
    outs: [
      '../book-links/cluster-loc-first-authors.parquet',
    ],
  },

  'cluster-first-authors': {
    cmd: 'duckscript cluster-first-authors.sql',
    deps: [
      'cluster-first-authors.sql',
      'cluster-ol-first-authors.parquet',
      'cluster-loc-first-authors.parquet',
    ],
    outs: [
      'cluster-first-authors.parquet',
    ],
  },

  'cluster-genders': {
    wdir: '..',
    cmd: bd.cmd('cluster-gender -o book-links/cluster-genders.parquet -A book-links/cluster-first-authors.parquet'),
    deps: [
      'src/cli/cluster_gender',
      'book-links/cluster-stats.parquet',
      'book-links/cluster-first-authors.parquet',
      'viaf/author-name-index.parquet',
      'viaf/author-genders.parquet',
    ],
    outs: [
      'book-links/cluster-genders.parquet',
    ],
  },

  'gender-stats': {
    wdir: '..',
    cmd: 'python book-links/integration-stats.py',
    deps: std.prune([
      'src/cli/stats.rs',
      'book-links/cluster-genders.parquet',
      'book-links/isbn-clusters.parquet',
      'loc-mds/book-isbn-ids.parquet',
      bd.maybe(bd.config.bx.enabled, 'bx/bx-cluster-actions.parquet'),
      bd.maybe(bd.config.bx.enabled, 'bx/bx-cluster-ratings.parquet'),
      bd.maybe(bd.config.az2014.enabled, 'az2014/az-cluster-ratings.parquet'),
      bd.maybe(bd.config.az2018.enabled, 'az2018/az-cluster-ratings.parquet'),
      bd.maybe(bd.config.goodreads.enabled, 'goodreads/gr-cluster-actions.parquet'),
      bd.maybe(bd.config.goodreads.enabled, 'goodreads/gr-cluster-ratings.parquet'),
    ]),
    outs: [
      'book-links/gender-stats.csv',
    ],
  },

  'cluster-hashes': {
    cmd: 'duckscript cluster-hashes.sql',
    deps: [
      'cluster-hashes.sql',
      'isbn-clusters.parquet',
    ],
    outs: [
      'cluster-hashes.parquet',
    ],
  },
})
