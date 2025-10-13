COPY (
    SELECT reviewerID AS user_id, asin, overall::FLOAT AS rating,
        make_timestamp_ms(unixReviewTime * 1000) AS timestamp,
        summary,
        reviewText AS text,
    FROM '../data/az2018/Books.json.gz'
) TO 'ratings.parquet' (COMPRESSION zstd);

COPY (
    SELECT user_id, asin, rating, timestamp
    FROM 'ratings.parquet'
) TO 'reviews.parquet' (COMPRESSION zstd);
