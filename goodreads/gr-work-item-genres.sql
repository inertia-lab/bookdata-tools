CREATE VIEW ids AS SELECT * FROM 'gr-book-ids.parquet';
CREATE VIEW bgs AS SELECT * FROM 'gr-book-genres.parquet';

COPY (
    SELECT item_id, genre_id, SUM(count) AS count
    FROM ids JOIN bgs USING (book_id)
    GROUP BY item_id, genre_id
)
TO 'gr-work-item-genres.parquet' (COMPRSSION ZSTD);
