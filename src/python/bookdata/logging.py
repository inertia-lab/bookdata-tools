from __future__ import annotations

import logging

from rich.console import Console
from rich.logging import RichHandler


def setup_logging(verbose: bool = False, *, lenskit: bool = False):
    if lenskit:
        from lenskit.logging import LoggingConfig

        lc = LoggingConfig()
        if verbose:
            lc.set_verbose(True)
        lc.apply()
        return

    level = logging.DEBUG if verbose else logging.INFO
    stderr = Console(stderr=True)
    root = logging.getLogger()
    root.setLevel(level)
    root.addHandler(RichHandler(console=stderr))
