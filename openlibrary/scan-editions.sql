CREATE MACRO key_id(key) AS regexp_replace(key, '^/\w+/OL(\d+)[A-Z]$', '\1')::INT32;

CREATE TABLE ol_editions AS
SELECT key_id(key) AS id, key, JSON record
FROM read_csv(
    '../data/openlib/ol_dump_editions.txt.gz',
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
    SELECT id, key, record->>'$.title' AS title
    FROM ol_editions
) TO 'editions.parquet' (COMPRESSION zstd);

COPY (
    SELECT id AS edition, key_id(UNNEST(record->>'$.works[*].key')) AS work
    FROM ol_editions
    WHERE json_exists(record, '$.works[*]')
) TO 'edition-works.parquet';

COPY (
    SELECT id AS edition, key_id(UNNEST(record->>'$.works[*].key')) AS work
    FROM ol_editions
    WHERE json_exists(record, '$.works[*]')
) TO 'edition-works.parquet';

COPY (
    SELECT id,
        generate_subscripts((record->>'$.authors[*]'), 1)::INT16 - 1AS pos,
        key_id(UNNEST(record->>'$.authors[*].author.key')) AS author
    FROM ol_editions
    WHERE record->>'$.authors[*]' IS NOT NULL
) TO 'edition-authors.parquet' (COMPRESSION zstd);

COPY (
    SELECT id, subj_type, COALESCE(subject->>'$.value', subject) AS subject
    FROM (
        SELECT id, 'G' AS subj_type, UNNEST(record->'$.subjects[*]') AS subject
        FROM ol_editions
        WHERE json_exists(record, '$.subjects')
        UNION ALL
        SELECT id, 'P', UNNEST(record->'$.subject_people[*]')
        FROM ol_editions
        WHERE json_exists(record, '$.subject_people')
        UNION ALL
        SELECT id, 'L', UNNEST(record->'$.subject_places[*]')
        FROM ol_editions
        WHERE json_exists(record, '$.subject_places')
        UNION ALL
        SELECT id, 'T', UNNEST(record->'$.subject_times[*]')
        FROM ol_editions
        WHERE json_exists(record, '$.subject_times')
    )
) TO 'edition-subjects.parquet';
