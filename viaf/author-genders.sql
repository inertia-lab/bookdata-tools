CREATE VIEW viaf AS SELECT * FROM 'viaf.parquet';

COPY (
    SELECT rec_id, lower(trim(contents))
    FROM viaf
    WHERE tag = 375
    AND sf_code = ascii('a')
) TO 'author-genders.parquet' (COMPRESSION zstd);
