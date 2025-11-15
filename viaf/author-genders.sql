SET enable_progress_bar = true;
CREATE VIEW viaf AS SELECT * FROM 'viaf.parquet';

COPY (
    SELECT rec_id, lower(trim(contents)) AS gender
    FROM viaf
    WHERE tag = 375
    AND sf_code = ascii('a')
) TO 'author-genders.parquet' (COMPRESSION zstd);
