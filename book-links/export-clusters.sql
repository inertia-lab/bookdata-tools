CREATE TABLE bg_nodes AS SELECT * FROM 'book-graph-nodes.parquet';

COPY (
    SELECT isbn_code(code) AS isbn_id, cluster
    FROM bg_nodes
    WHERE type = 'ISBN'
    ORDER BY isbn_id
) TO 'isbn-clusters.parquet' (COMPRESSION zstd);

COPY (
    SELECT cluster,
        COUNT(code) AS n_nodes,
        COUNT(code) FILTER (type = 'ISBN') AS n_isbns,
        COUNT(code) FILTER (type = 'LOC') AS n_loc_recs,
        COUNT(code) FILTER (type = 'OL-E') AS n_ol_editions,
        COUNT(code) FILTER (type = 'OL-W') AS n_ol_works,
        COUNT(code) FILTER (type = 'GR-B') AS n_gr_editions,
        COUNT(code) FILTER (type = 'GR-W') AS n_gr_works,
    FROM bg_nodes
    GROUP BY cluster
) TO 'cluster-stats.parquet' (COMPRESSION zstd);
