-- Extract GoodReads book links.
CREATE VIEW book_nodes AS
SELECT gr_book_id(code) AS book_id,  cluster
FROM '../book-links/book-graph-nodes.parquet'
WHERE type = 'GR-B';

COPY (
    SELECT book_id, work_id, item_id, cluster
    FROM book_nodes
    LEFT JOIN 'gr-book-ids.parquet' USING (book_id)
) TO 'gr-book-link.parquet' (COMPRESSION zstd);
