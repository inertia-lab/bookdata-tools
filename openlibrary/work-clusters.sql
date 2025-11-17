-- Extract GoodReads book links.
CREATE VIEW book_nodes AS
SELECT ol_work_id(code) AS work_id, cluster
FROM '../book-links/book-graph-nodes.parquet'
WHERE type = 'OL-W';

COPY book_nodes TO 'work-clusters.parquet' (COMPRESSION zstd);
