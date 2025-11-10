COPY (
    SELECT cluster,
        md5(encode(string_agg(isbn, '' ORDER BY isbn))) AS isbn_hash
    FROM 'isbn-clusters.parquet'
    GROUP BY cluster
) TO 'cluster-hashes.parquet' (COMPRESSION zstd);
