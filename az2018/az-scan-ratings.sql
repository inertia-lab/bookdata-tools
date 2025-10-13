COPY (
    SELECT user_id, asin, rating::FLOAT AS rating,
        make_timestamp_ms(timestamp * 1000) AS timestamp
    FROM read_csv(
        '../data/az2018/Books.csv',
        names=['asin', 'user_id', 'rating', 'timestamp']
    )
) TO 'ratings.parquet' (COMPRESSION zstd);
