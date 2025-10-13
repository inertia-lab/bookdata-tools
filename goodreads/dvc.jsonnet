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
    wdir: '..',
    cmd: bd.cmd('cluster extract-books -o goodreads/gr-book-link.parquet -n book_id --join-file goodreads/gr-book-ids.parquet --join-field work_id GR-B'),
    deps: [
      'goodreads/gr-book-ids.parquet',
      'book-links/cluster-graph-nodes.parquet',
    ],
    outs: [
      'goodreads/gr-book-link.parquet',
    ],
  },
};

bd.pipeline(scan + links + clusters + works, bd.config.goodreads.enabled)
