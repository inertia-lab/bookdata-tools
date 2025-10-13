CREATE VIEW isbns AS SELECT isbn, cluster FROM '../book-links/isbn-clusters.parquet';
CREATE VIEW ratings AS SELECT * FROM 'ratings.parquet';

COPY (
    SELECT user_id, cluster AS item_id,
        median(rating) as rating,
        last(rating ORDER BY timestamp) AS last_rating,
        min(timestamp) AS first_time,
        max(timestamp) AS last_time,
        count(cluster) AS nratings
    FROM isbns JOIN ratings ON (asin = isbn)
    GROUP BY user_id, cluster
) TO 'az-cluster-ratings.parquet';
