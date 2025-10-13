CREATE VIEW isbns AS SELECT isbn, cluster FROM '../book-links/isbn-clusters.parquet';
CREATE VIEW ratings AS SELECT * FROM 'cleaned-ratings.csv';

COPY (
    SELECT user AS user_id, cluster AS item_id,
        median(rating) AS rating,
        count(rating) AS nratings
    FROM ratings JOIN isbns USING (isbn)
    GROUP BY user_id, item_id
) TO 'bx-cluster-ratings.parquet' (COMPRESSION zstd);
