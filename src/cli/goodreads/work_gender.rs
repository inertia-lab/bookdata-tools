use crate::{
    ids::codes::{NS_GR_BOOK, NS_GR_WORK},
    prelude::*,
};
use polars::prelude::*;

pub fn link_work_genders() -> Result<()> {
    require_working_dir("goodreads")?;

    let gender = LazyFrame::scan_parquet("../book-links/cluster-genders.parquet", default())?;
    let books = LazyFrame::scan_parquet("gr-book-link.parquet", default())?;

    let merged = gender.join(
        books,
        &[col("cluster")],
        &[col("cluster")],
        JoinType::Inner.into(),
    );
    let dedup = merged.unique(None, UniqueKeepStrategy::First);
    let dedup = dedup.select([
        col("*"),
        coalesce(&[
            col("work_id") + lit(NS_GR_WORK.base()),
            col("book_id") + lit(NS_GR_BOOK.base()),
        ])
        .alias("item_id"),
    ]);

    info!("computing book genders");
    let results = dedup.clone().collect()?;

    info!("saving {} book-gender records", results.height());
    save_df_parquet(results, "gr-book-gender.parquet")?;

    info!("computing item genders");
    let dd2 = dedup
        .select(&[col("item_id"), col("gender")])
        .unique(Some(vec!["item_id".into()]), UniqueKeepStrategy::First);
    let results = dd2.collect()?;

    info!("saving {} item-gender records", results.height());
    save_df_parquet(results, "gr-work-item-gender.parquet")?;

    Ok(())
}
