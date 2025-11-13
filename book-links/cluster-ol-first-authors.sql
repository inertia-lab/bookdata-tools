CREATE VIEW icl AS SELECT isbn_id, cluster FROM 'isbn-clusters.parquet';
CREATE VIEW ol_edl AS
SELECT isbn_id, edition FROM '../openlibrary/edition-isbn-ids.parquet'
WHERE isbn_id IS NOT NULL;
CREATE VIEW ol_eau AS
SELECT * FROM '../openlibrary/edition-authors.parquet'
WHERE pos = 0;

CREATE VIEW ol_auth AS SELECT * FROM '../openlibrary/author-names.parquet';

CREATE VIEW ol_authors AS
SELECT DISTINCT
    cluster,
    TRIM(regexp_replace(
        regexp_replace(ol_auth.name, '[\s\.]+', ' ', 'g'),
        '\s+,', ',', 'g'
     )) COLLATE NFC AS author_name
FROM icl
JOIN ol_edl USING (isbn_id)
JOIN ol_eau USING (edition)
JOIN ol_auth ON (ol_eau.author = ol_auth.id)
WHERE author_name IS NOT NULL AND author_name <> '';

COPY ol_authors TO 'cluster-ol-first-authors.parquet' (COMPRESSION zstd);
