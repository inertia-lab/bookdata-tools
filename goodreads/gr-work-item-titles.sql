CREATE VIEW ids AS SELECT * FROM 'gr-book-ids.parquet';
CREATE VIEW book_info AS SELECT book_id, title FROM 'gr-book-info.parquet';
CREATE VIEW work_info AS SELECT work_id, title FROM 'gr-work-info.parquet';

CREATE TABLE works AS
SELECT DISTINCT item_id, work_id, NULL AS book_id
FROM ids
WHERE work_id IS NOT NULL;

CREATE TABLE books AS
SELECT DISTINCT item_id, work_id, book_id
FROM ids
WHERE work_id IS NULL;

COPY (
    SELECT item_id, work_id, book_id, title
    FROM books LEFT JOIN book_info USING (book_id)
    UNION DISTINCT
    SELECT item_id, work_id, book_id, title
    FROM works LEFT JOIN work_info USING (work_id)
)
TO 'gr-work-item-titles.parquet' (COMPRESSION ZSTD);
