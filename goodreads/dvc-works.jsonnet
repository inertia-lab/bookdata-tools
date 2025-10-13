local bd = import '../bookdata.libsonnet';

{
  'work-item-titles': {
    cmd: 'python ../scripts/duckscript.py gr-work-item-titles.sql',
    deps: [
      'gr-work-item-titles.sql',
      'gr-book-ids.parquet',
      'gr-book-info.parquet',
      'gr-work-info.parquet',
    ],
    outs: [
      'gr-work-item-titles.parquet',
    ],
  },

  'work-item-genres': {
    cmd: 'python ../scripts/duckscript.py gr-work-item-genres.sql',
    deps: [
      'gr-work-item-genres.sql',
      'gr-book-ids.parquet',
      'gr-book-genres.parquet',
    ],
    outs: [
      'gr-work-item-genres.parquet',
    ],
  },

  'work-actions': {
    wdir: '..',
    cmd: bd.cmd('goodreads cluster-interactions --add-actions --native-works -o goodreads/gr-work-actions.parquet'),
    deps: [
      'src/cli/goodreads/cluster.rs',
      'goodreads/gr-book-link.parquet',
      'goodreads/gr-interactions.parquet',
    ],
    outs: [
      'goodreads/gr-work-actions.parquet',
    ],
  },

  'work-ratings': {
    wdir: '..',
    cmd: bd.cmd('goodreads cluster-interactions --ratings --native-works -o goodreads/gr-work-ratings.parquet'),
    deps: [
      'src/cli/goodreads/cluster.rs',
      'goodreads/gr-book-link.parquet',
      'goodreads/gr-interactions.parquet',
    ],
    outs: [
      'goodreads/gr-work-ratings.parquet',
    ],
  },

  'work-ratings-5core': {
    cmd: 'python ../scripts/kcore.py -o gr-work-ratings-5core.parquet gr-work-ratings.parquet',
    deps: [
      '../scripts/kcore.py',
      'gr-work-ratings.parquet',
    ],
    outs: [
      'gr-work-ratings-5core.parquet',
    ],
  },

  'work-actions-5core': {
    cmd: 'python ../scripts/kcore.py -o gr-work-actions-5core.parquet gr-work-actions.parquet',
    deps: [
      '../scripts/kcore.py',
      'gr-work-actions.parquet',
    ],
    outs: [
      'gr-work-actions-5core.parquet',
    ],
  },

  'work-ratings-2015-100-10core': {
    cmd: 'python ../scripts/kcore.py --user-k 10 --item-k 100 --year 2015 -o gr-work-ratings-2015-100-10core.parquet gr-work-ratings.parquet',
    deps: [
      'gr-work-ratings.parquet',
      '../scripts/kcore.py',
    ],
    outs: [
      'gr-work-ratings-2015-100-10core.parquet',
    ],
  },

  'work-actions-2015-100-10core': {
    cmd: 'python ../scripts/kcore.py --user-k 10 --item-k 100 --year 2015 -o gr-work-actions-2015-100-10core.parquet gr-work-actions.parquet',
    deps: [
      'gr-work-actions.parquet',
      '../scripts/kcore.py',
    ],
    outs: [
      'gr-work-actions-2015-100-10core.parquet',
    ],
  },

  'works-dataset': {
    cmd: 'python gr-dataset.py --works gr-works-dataset',
    deps: [
      'gr-dataset.py',
      'gr-work-actions.parquet',
      'gr-genres.parquet',
      'gr-work-item-genres.parquet',
      'gr-work-item-titles.parquet',
    ],
    outs: [
      'gr-works-dataset',
    ],
  },

  'works-5core-dataset': {
    cmd: 'python gr-dataset.py --works --core gr-works-5core-dataset',
    deps: [
      'gr-dataset.py',
      'gr-work-actions-5core.parquet',
      'gr-genres.parquet',
      'gr-work-item-genres.parquet',
      'gr-work-item-titles.parquet',
    ],
    outs: [
      'gr-works-5core-dataset',
    ],
  },

  'work-author': {
    cmd: 'python gr-work-item-authors.py',
    deps: [
      'gr-work-item-authors.py',
      'gr-book-ids.parquet',
      'gr-book-authors.parquet',
      'gr-author-info.parquet',
    ],
    outs: [
      'gr-work-item-authors.parquet',
    ],
  },

  'work-first-author': {
    cmd: 'python gr-work-item-authors.py --first',
    deps: [
      'gr-work-item-authors.py',
      'gr-book-ids.parquet',
      'gr-book-authors.parquet',
      'gr-author-info.parquet',
    ],
    outs: [
      'gr-work-item-first-authors.parquet',
    ],
  },

  'work-gender': {
    cmd: 'python ../scripts/duckscript.py gr-work-item-gender.sql',
    deps: [
      'gr-work-item-gender.sql',
      'gr-book-link.parquet',
      '../book-links/cluster-genders.parquet',
    ],
    outs: [
      'gr-book-gender.parquet',
      'gr-work-item-gender.parquet',
    ],
  },

  'work-all-actions': {
    cmd: 'python ../scripts/duckscript.py gr-work-all-actions.sql',
    deps: [
      'gr-interactions.parquet',
      'gr-book-ids.parquet',
      'gr-work-all-actions.sql',
    ],
    outs: [
      'gr-work-all-actions.parquet',
    ],
  },

  'work-actions-15core': {
    cmd: 'python ../scripts/kcore.py --user-k 15 --item-k 15 -o gr-work-actions-15core.parquet gr-work-actions.parquet',
    deps: [
      'gr-work-actions.parquet',
      '../scripts/kcore.py',
    ],
    outs: [
      'gr-work-actions-15core.parquet',
    ],
  },
}
