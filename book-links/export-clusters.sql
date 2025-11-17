COPY (
    SELECT code, cluster, type, label
    FROM bg_nodes
    JOIN node_clusters USING (code)
    ORDER BY code
) TO 'book-graph-nodes.parquet' (COMPRESSION zstd);
COPY bg_edges TO 'book-graph-edges.parquet' (COMPRESSION zstd);

COPY (
    SELECT isbn_code(code) AS isbn_id, cluster, label AS isbn
    FROM bg_nodes
    JOIN node_clusters USING (code)
    WHERE type = 'ISBN'
    ORDER BY isbn_id
) TO 'isbn-clusters.parquet' (COMPRESSION zstd);

COPY cluster_stats TO 'cluster-stats.parquet' (COMPRESSION zstd);
