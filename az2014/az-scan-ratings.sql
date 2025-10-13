COPY (
    SELECT user_id, asin, rating::FLOAT AS rating,
        make_timestamp_ms(timestamp * 1000) AS timestamp
    FROM read_csv(
        '../data/az2014/ratings_Books.csv',
        names=['user_id', 'asin', 'rating', 'timestamp']
    )
) TO 'ratings.parquet'  (COMPRESSION zstd);
