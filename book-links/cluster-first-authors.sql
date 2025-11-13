CREATE VIEW ol_authors AS
SELECT * FROM 'cluster-ol-first-authors.parquet';

CREATE VIEW loc_authors AS
SELECT * FROM 'cluster-loc-first-authors.parquet';

COPY (
    SELECT cluster, author_name FROM ol_authors
    UNION DISTINCT
    SELECT cluster, author_name FROM loc_authors
) TO 'cluster-first-authors.parquet' (COMPRESSION zstd);
