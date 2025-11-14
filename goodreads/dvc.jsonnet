local bd = import '../bookdata.libsonnet';

local clusters = import 'dvc-clusters.jsonnet';
local scan = import 'dvc-scan.jsonnet';
local works = import 'dvc-works.jsonnet';
local links = {
  'book-isbn-ids': {
    wdir: '..',
    cmd: 'python scripts/link-isbns.py -o goodreads/book-isbn-ids.parquet -R book_id -I isbn10 -I isbn13 -I asin goodreads/gr-book-ids.parquet',
    deps: [
      'scripts/link-isbns.py',
      'goodreads/gr-book-ids.parquet',
      'book-links/isbns.db',
    ],
    outs: [
      'goodreads/book-isbn-ids.parquet',
    ],
  },

  'book-links': {
    cmd: 'duckscript gr-book-link.sql',
    deps: [
      'gr-book-link.sql',
      'gr-book-ids.parquet',
      '../book-links/cluster-graph-nodes.parquet',
    ],
    outs: [
      'gr-book-link.parquet',
    ],
  },
};

bd.pipeline(scan + links + clusters + works, bd.config.goodreads.enabled)
