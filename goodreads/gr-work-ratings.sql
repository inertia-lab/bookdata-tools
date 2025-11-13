CREATE VIEW actions AS
SELECT user_id,
    COALESCE(gr_work_code(work_id), gr_book_code(book_id)) AS item_id,
    updated AS timestamp, rating
FROM 'gr-interactions.parquet' gri
JOIN 'gr-book-link.parquet' grbl USING (book_id)
WHERE rating IS NOT NULL;

COPY (
    SELECT user_id, item_id,
        ARG_MAX(rating, timestamp) AS rating,
        MIN(timestamp) AS first_time,
        MAX(timestamp) AS last_time,
        COUNT(item_id) AS nactions,
    FROM actions
    GROUP BY user_id, item_id
    ORDER BY first_time
) TO 'gr-work-ratings.parquet' (COMPRESSION zstd);
