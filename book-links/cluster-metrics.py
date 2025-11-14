import json

from duckdb import connect

with connect() as db:
    db.execute(
        """
        SELECT SUM(n_nodes), COUNT(*), MAX(n_nodes), MAX(n_isbns)
        FROM 'cluster-stats.parquet'
        """
    )
    nodes, clusters, largest, max_isbns = db.fetchone() or []
    with open("cluster-metrics.json", "wt") as jsf:
        print(
            json.dumps(
                {
                    "nodes": nodes,
                    "clusters": clusters,
                    "largest": largest,
                    "max_isbns": max_isbns,
                }
            ),
            file=jsf,
        )
