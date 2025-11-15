CREATE MACRO key_id(key) AS regexp_replace(key, '^/\w+/OL(\d+)[A-Z]$', '\1')::INT32;
CREATE MACRO ol_text_value(val) AS CASE
    WHEN json_exists(val, '$.value') THEN val->>'$.value'
    ELSE val->>'$'
END;

--! Load work data
CREATE TABLE ol_works AS
SELECT key_id(key) AS id, key,
    record->>'title' AS title,
    record->'$.authors[*]' AS authors,
    record->'$.subjects[*]' AS subjects,
    record->'$.subject_people[*]' AS subject_people,
    record->'$.subject_places[*]' AS subject_places,
    record->'$.subject_times[*]' AS subject_times
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
    SELECT id, key, title
    FROM ol_works
) TO 'works.parquet' (COMPRESSION zstd);

--! Collect work authors
COPY (
    SELECT id,
        generate_subscripts(authors->'$[*]', 1)::INT16 - 1 AS pos,
        key_id(UNNEST(authors->>'$[*].author.key')) AS author
    FROM ol_works
    WHERE authors IS NOT NULL
) TO 'work-authors.parquet' (COMPRESSION zstd);

CREATE TEMPORARY TABLE work_subjects (
    id INTEGER NOT NULL,
    subj_type CHAR(1) NOT NULL,
    subject VARCHAR NOT NULL,
);

--! Collect general subjects
INSERT INTO work_subjects (id, subj_type, subject)
SELECT id, 'G' AS subj_type, ol_text_value(UNNEST(subjects)) AS subject
FROM ol_works
WHERE subjects IS NOT NULL;

--! Collect people subjects
INSERT INTO work_subjects (id, subj_type, subject)
SELECT id, 'P', ol_text_value(UNNEST(subject_people))
FROM ol_works
WHERE subject_people IS NOT NULL;

--! Collect place subjects
INSERT INTO work_subjects (id, subj_type, subject)
SELECT id, 'L', ol_text_value(UNNEST(subject_places))
FROM ol_works
WHERE subject_places IS NOT NULL;

--! Collect time subjects
INSERT INTO work_subjects (id, subj_type, subject)
SELECT id, 'T', ol_text_value(UNNEST(subject_times))
FROM ol_works
WHERE subject_times IS NOT NULL;

--! Export compiled subjects
COPY (
    SELECT * FROM work_subjects ORDER BY id, subj_type
) TO 'work-subjects.parquet';
