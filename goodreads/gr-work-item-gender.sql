CREATE TABLE genders AS
SELECT DISTINCT bl.*, gender,
    COALESCE(gr_work_code(work_id), gr_book_code(book_id)) AS item_id
FROM '../book-links/cluster-genders.parquet' cg
JOIN 'gr-book-link.parquet' bl USING (cluster);

COPY genders TO 'gr-book-gender.parquet';
COPY (
    SELECT DISTINCT ON (item_id) item_id, gender
    FROM genders
) TO 'gr-work-item-gender.parquet';
