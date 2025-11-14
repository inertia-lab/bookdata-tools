-- -n book_id
-- --join-file goodreads/gr-book-ids.parquet --join-field work_id GR-B

CREATE VIEW book_nodes AS
SELECT gr_book_id(book_code) AS book_id,  cluster
FROM '../book-links/cluster-graph-nodes.parquet'
WHERE is_gr_book(book_code);

COPY (
    SELECT book_id,  work_id, cluster
    FROM book_nodes
    LEFT JOIN 'gr-book-ids.parquet' USING (book_id)
) TO 'gr-book-link.parquet' (COMPRESSION zstd);
