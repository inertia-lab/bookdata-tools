local bd = import '../bookdata.libsonnet';

bd.pipeline({
  'scan-authors': {
    cmd: bd.cmd('scan-marc -L -o viaf.parquet ../data/viaf-clusters-marc21.xml.gz'),
    deps: [
      '../src/cli/scan_marc.rs',
      '../src/marc',
      '../data/viaf-clusters-marc21.xml.gz',
    ],
    outs: [
      'viaf.parquet',
    ],
  },
  'author-genders': {
    cmd: 'duckscript author-genders.sql',
    deps: [
      'author-genders.sql',
      'viaf.parquet',
    ],
    outs: [
      'author-genders.parquet',
    ],
  },
  'index-names': {
    cmd: bd.cmd('index-names --marc-authorities viaf.parquet author-name-index.parquet'),
    deps: [
      '../src/cli/index_names.rs',
      '../src/cleaning/names',
      'viaf.parquet',
    ],
    outs: [
      'author-name-index.parquet',
      'author-name-index.csv.gz',
    ],
  },
})
