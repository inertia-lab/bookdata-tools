# Overview {.unnumbered}

The PIReT/INERTIAL Book Data Tools are a set of tools for ingesting, integrating, and
indexing a variety of sources of book data, originally created by the [People
and Information Research Team](https://piret.info) at [Boise State
University](https://boisestate.edu) and now maintained by the [INERTIA
Lab](https://inertial.science) at [Drexel University](https://drexel.edu/cci/).
The result of running these tools is a set of Parquet files with raw data in a
more usable form, various useful extracted features, and integrated identifiers
across the various data sources for cross-linking.  These tools are updated from
the version used to support our original paper; we have dropped PostgreSQL in
favor of a pipeline using DVC to script extraction and integration tools
implemented in Rust that is more efficient (integration times have dropped from
8 hours to less than 3) and requires significantly less disk space.[^bftag]

The latest version of these tools is **3.0** (see [history](history.md)).

[^bftag]: The original tools are available on the `before-fusion` tag in the Git repository.

If you use these scripts in any published research, cite [our paper][paper] ([PDF][]):

[paper]: https://md.ekstrandom.net/pubs/bag-extended
[PDF]: https://md.ekstrandom.net/pubs/bag2-preprint.pdf

> Michael D. Ekstrand and Daniel Kluver. 2021. Exploring Author Gender in Book Rating and Recommendation. <cite>User Modeling and User-Adapted Interaction</cite> (February 2021) DOI:[10.1007/s11257-020-09284-2](https://doi.org/10.1007/s11257-020-09284-2).

We also ask that you contact Michael Ekstrand to let us know about your use of the data, so we can
include your paper in our list of relying publications.

::: callout-warning
The “Limitations” section of the paper contains important information about
the limitations of the data these scripts compile.  **Do not use the gender information
in this data or tools without understanding those limitations**.  In particular,
VIAF's gender information is incomplete and, in a number of cases, incorrect.
:::

In addition, several of the data sets integrated by this project come from other sources
with their own publications.  **If you use any of the rating or interaction data, cite the
appropriate original source paper.**  For each data set below, we have provided a link to the
page that describes the data and its appropriate citation.

See the [Setup page](using/setup.md) to get started and for system requirements.

## Video

I recorded a video walking through the integration as an example for my [Data Science class](https://cs533.ekstrandom.net).
This discusses the PostgreSQL version of the integration, but the concepts have remained the same in terms of linking logic.

<div style="position: relative; padding-top: 56.25%;"><iframe src="https://iframe.mediadelivery.net/embed/114107/60f24ed2-4a71-4fa7-b1f4-82d95ab62c67?autoplay=false" loading="lazy" style="border: none; position: absolute; top: 0; height: 100%; width: 100%;" allow="accelerometer; gyroscope; autoplay; encrypted-media; picture-in-picture;" allowfullscreen="true"></iframe></div>

## License

These tools are under the MIT license:

> Copyright 2019-2021 Boise State University
>
> Permission is hereby granted, free of charge, to any person obtaining a copy of
> this software and associated documentation files (the "Software"), to deal in
> the Software without restriction, including without limitation the rights to
> use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
> the Software, and to permit persons to whom the Software is furnished to do so,
> subject to the following conditions:
>
> The above copyright notice and this permission notice shall be included in all
> copies or substantial portions of the Software.
>
> THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
> IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
> FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
> COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
> IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
> CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

## Acknowledgments

This material is based upon work supported by the National Science Foundation
under Grant No. [IIS 17-51278](https://md.ekstrandom.net/research/career). Any
opinions, findings, and conclusions or recommendations expressed in this
material are those of the author(s) and do not necessarily reflect the views of
the National Science Foundation.  This page has not been approved by Boise State
University and does not reflect official university positions.
