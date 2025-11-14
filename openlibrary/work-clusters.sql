-- Extract GoodReads book links.
CREATE VIEW book_nodes AS
SELECT ol_work_id(book_code) AS work_id,  cluster
FROM '../book-links/cluster-graph-nodes.parquet'
WHERE is_ol_work(book_code);

COPY book_nodes TO 'work-clusters.parquet' (COMPRESSION zstd);
