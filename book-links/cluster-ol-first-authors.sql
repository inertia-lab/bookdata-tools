CREATE VIEW icl AS SELECT isbn_id, cluster FROM 'isbn-clusters.parquet';
CREATE VIEW ol_edl AS SELECT isbn_id, edition FROM '../openlibrary/edition-isbn-ids.parquet';
CREATE VIEW ol_eau AS
SELECT * FROM '../openlibrary/edition-authors.parquet'
WHERE pos = 0;

CREATE VIEW ol_auth AS SELECT * FROM '../openlibrary/author-names.parquet';

CREATE VIEW ol_authors AS
SELECT DISTINCT cluster, ol_auth.name AS author_name
FROM icl
JOIN ol_edl USING (isbn_id)
JOIN ol_eau USING (edition)
JOIN ol_auth ON (ol_eau.author = ol_auth.id)
WHERE name IS NOT NULL AND name <> '';

COPY ol_authors TO 'cluster-ol-first-authors.parquet' (COMPRESSION zstd);
