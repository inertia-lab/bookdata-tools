CREATE VIEW icl AS SELECT isbn_id, cluster FROM 'isbn-clusters.parquet';
CREATE VIEW loc_books AS SELECT * FROM '../loc-mds/book-isbn-ids.parquet';
CREATE VIEW loc_auth AS
SELECT * FROM '../loc-mds/book-authors.parquet'
WHERE author_name IS NOT NULL;

CREATE VIEW loc_authors AS
SELECT DISTINCT cluster, author_name
FROM icl
JOIN loc_books USING (isbn_id)
JOIN loc_auth USING (rec_id);

COPY loc_authors TO 'cluster-loc-first-authors.parquet' (COMPRESSION zstd);
