SET enable_progress_bar = true;

-- clustering is fun!
CREATE TEMPORARY TABLE bg_clusters AS
WITH RECURSIVE clusters(code, cluster, size, depth) USING KEY (code) AS (
    -- base case: initialize the table with the nodes
    SELECT code, code, 1, 0 FROM bg_nodes
    UNION
    -- recursive case: update table with nodes
    SELECT upd.code, upd.cluster,
        COUNT(upd.code) OVER (PARTITION BY upd.cluster),
        upd.depth + 1
    FROM (
        -- find new cluster IDs for each node in the last round
        SELECT lc.code AS code, lc.depth AS depth, lc.size AS size,
            LEAST(lc.cluster, MIN(rc.cluster)) AS cluster
        FROM clusters lc
        JOIN bg_neighbors n ON lc.code = n.node
        JOIN recurring.clusters rc ON n.nbr = rc.code
        GROUP BY lc.code, lc.depth, lc.size, lc.cluster
    ) upd
    -- keep the ones that are actually a change
    QUALIFY COUNT(upd.code) OVER (PARTITION BY upd.cluster) > upd.size
)
SELECT code, cluster, size, depth FROM clusters;

COPY bg_clusters TO 'book-graph-clusters.parquet' (COMPRESSION zstd);
