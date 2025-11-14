CREATE MACRO clean_name(name) AS
    TRIM(regexp_replace(
        regexp_replace(name, '[\s\.]+', ' ', 'g'),
        '\s+,', ',', 'g'
     ));

CREATE VIEW ol_authors AS
SELECT regexp_replace(key, '^/\w+/OL(\d+)[A-Z]$', '\1')::INT32 AS id, key, record
FROM read_csv(
    '../data/openlib/ol_dump_authors.txt.gz',
    delim='\t',
    columns={
        'type': 'VARCHAR',
        'key': 'VARCHAR',
         'version': 'INTEGER',
         'date': 'VARCHAR',
         'record': 'JSON'
    }
);

COPY (
    SELECT id, key, record->>'name' AS name
    FROM ol_authors
) TO 'authors.parquet' (COMPRESSION zstd);

COPY (
    SELECT id, 'n' AS source, clean_name(record->>'$.name') AS name
    FROM ol_authors
    WHERE record->>'name' IS NOT NULL
    UNION ALL
    SELECT id, 'p', clean_name(record->>'$.personal_name')
    FROM ol_authors
    WHERE record->>'personal_name' IS NOT NULL
    UNION ALL
    SELECT id, 'a', clean_name(UNNEST(record->>'$.alternate_names[*]'))
    FROM ol_authors
    WHERE record->>'alternate_names' IS NOT NULL
) TO 'author-names.parquet' (COMPRESSION zstd);
