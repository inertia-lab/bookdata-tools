local bd = import '../bookdata.libsonnet';

{
  'work-item-titles': {
    cmd: 'python gr-work-item-titles.py',
    deps: [
      'gr-work-item-titles.py',
      'gr-book-ids.parquet',
      'gr-book-info.parquet',
      'gr-work-info.parquet',
    ],
    outs: [
      'gr-work-item-titles.parquet',
    ],
  },

  'work-item-genres': {
    cmd: 'python gr-work-item-genres.py',
    deps: [
      'gr-work-item-genres.py',
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
    cmd: bd.cmd('kcore -o gr-work-ratings-5core.parquet gr-work-ratings.parquet'),
    deps: [
      '../src/cli/kcore.rs',
      'gr-work-ratings.parquet',
    ],
    outs: [
      'gr-work-ratings-5core.parquet',
    ],
  },

  'work-actions-5core': {
    cmd: bd.cmd('kcore -o gr-work-actions-5core.parquet gr-work-actions.parquet'),
    deps: [
      '../src/cli/kcore.rs',
      'gr-work-actions.parquet',
    ],
    outs: [
      'gr-work-actions-5core.parquet',
    ],
  },

  'work-ratings-2015-100-10core': {
    cmd: bd.cmd('kcore --user-k 10 --item-k 100 --year 2015 -o gr-work-ratings-2015-100-10core.parquet gr-work-ratings.parquet'),
    deps: [
      'gr-work-ratings.parquet',
      '../src/cli/kcore.rs',
    ],
    outs: [
      'gr-work-ratings-2015-100-10core.parquet',
    ],
  },

  'work-actions-2015-100-10core': {
    cmd: bd.cmd('kcore --user-k 10 --item-k 100 --year 2015 -o gr-work-actions-2015-100-10core.parquet gr-work-actions.parquet'),
    deps: [
      'gr-work-actions.parquet',
      '../src/cli/kcore.rs',
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
    cmd: bd.cmd('goodreads work-gender'),
    deps: [
      '../src/cli/goodreads',
      'gr-book-link.parquet',
      '../book-links/cluster-genders.parquet',
    ],
    outs: [
      'gr-book-gender.parquet',
      'gr-work-item-gender.parquet',
    ],
  },

  'work-all-actions': {
    cmd: 'python gr-work-all-actions.py',
    deps: [
      'gr-interactions.parquet',
      'gr-book-ids.parquet',
      'gr-work-all-actions.py',
    ],
    outs: [
      'gr-work-all-actions.parquet',
    ],
  },

  'work-actions-15core': {
    cmd: bd.cmd('kcore --user-k 15 --item-k 15 -o gr-work-actions-15core.parquet gr-work-actions.parquet'),
    deps: [
      'gr-work-actions.parquet',
      '../src/cli/kcore.rs',
    ],
    outs: [
      'gr-work-actions-15core.parquet',
    ],
  },
}
