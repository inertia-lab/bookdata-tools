-- Extract all actions related to a work.
CREATE VIEW ids AS SELECT * FROM 'gr-book-ids.parquet';
CREATE VIEW actions AS SELECT * FROM 'gr-interactions.parquet';

COPY (
    SELECT user_id, item_id, book_id, work_id, rating, is_read,
    added AS timestamp,
    updated
    FROM ids
    JOIN actions USING ('book_id')
)
TO 'gr-work-all-actions.parquet' (COMPRESSION ZSTD);
