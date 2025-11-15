CREATE MACRO key_id(key) AS regexp_replace(key, '^/\w+/OL(\d+)[A-Z]$', '\1')::INT32;
CREATE MACRO ol_text_value(val) AS CASE
    WHEN json_exists(val, '$.value') THEN val->>'$.value'
    ELSE val->>'$'
END;

--! Load work data
CREATE TABLE ol_works AS
SELECT key_id(key) AS id, key, record::JSON AS record
FROM read_csv(
    '../data/openlib/ol_dump_works.txt.gz',
    delim='\t',
    columns={
        'type': 'VARCHAR',
        'key': 'VARCHAR',
         'version': 'INTEGER',
         'date': 'VARCHAR',
         'record': 'JSON'
    }
);

--! Collect work titles
COPY (
    SELECT id, key, record->>'$.title' AS title
    FROM ol_works
) TO 'works.parquet' (COMPRESSION zstd);

--! Collect work authors
COPY (
    SELECT id,
        generate_subscripts((record->>'$.authors[*]'), 1)::INT16 - 1 AS pos,
        key_id(UNNEST(record->>'$.authors[*].author.key')) AS author
    FROM ol_works
    WHERE record->>'$.authors[*]' IS NOT NULL
) TO 'work-authors.parquet' (COMPRESSION zstd);

CREATE TEMPORARY TABLE work_subjects (
    id INTEGER NOT NULL,
    subj_type CHAR(1) NOT NULL,
    subject VARCHAR NOT NULL
);

--! Collect general subjects
INSERT INTO work_subjects (id, subj_type, subject)
SELECT id, 'G' AS subj_type, ol_text_value(UNNEST(record->'$.subjects[*]')) AS subject
FROM ol_works
WHERE json_exists(record, '$.subjects');

--! Collect people subjects
INSERT INTO work_subjects (id, subj_type, subject)
SELECT id, 'P', ol_text_value(UNNEST(record->'$.subject_people[*]'))
FROM ol_works
WHERE json_exists(record, '$.subject_people');

--! Collect place subjects
INSERT INTO work_subjects (id, subj_type, subject)
SELECT id, 'L', ol_text_value(UNNEST(record->'$.subject_places[*]'))
FROM ol_works
WHERE json_exists(record, '$.subject_places');

--! Collect time subjects
INSERT INTO work_subjects (id, subj_type, subject)
SELECT id, 'T', ol_text_value(UNNEST(record->'$.subject_times[*]'))
FROM ol_works
WHERE json_exists(record, '$.subject_times');

--! Export compiled subjects
COPY (
    SELECT * FROM work_subjects ORDER BY id
) TO 'work-subjects.parquet';
