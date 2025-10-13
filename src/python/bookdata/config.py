from __future__ import annotations

import re
from pathlib import Path

import pyprojroot
import yaml
from pydantic import BaseModel

SRC_NAMES = {
    "BX": "bx",
    "AZ14": "az2014",
    "AZ18": "az2018",
    "GR": "goodreads",
}


class SourceConfig(BaseModel):
    """
    Configuration for a data source.
    """

    enabled: bool = True
    "Whether the data source is enabled."


class BookDataConfig(BaseModel):
    """
    Configuration for the book data tools.
    """

    bx: SourceConfig = SourceConfig()
    az2014: SourceConfig = SourceConfig()
    az2018: SourceConfig = SourceConfig()
    goodreads: SourceConfig = SourceConfig()

    @property
    def root(self) -> Path:
        return bd_root()

    @classmethod
    def load(cls) -> BookDataConfig:
        root = bd_root()
        file = root / "config.yaml"
        src = file.read_text()
        parsed = yaml.load(src, yaml.SafeLoader)
        return cls.model_validate(parsed)

    def source_enabled(self, name: str) -> bool:
        """
        Query whether a data source is enabled.
        """
        if m := re.match(r"^(\w+)-", name):
            name = m.group(1)

        match name:
            case "loc" | "LOC":
                return True
            case "openlib" | "openlibrary" | "OL":
                return True
            case "goodreads" | "GR":
                return self.goodreads.enabled
            case "az2014" | "AZ14":
                return self.az2014.enabled
            case "az2018" | "AZ18":
                return self.az2018.enabled
            case "bx" | "BX":
                return self.bx.enabled
            case _:
                raise ValueError(f"unknown data set {name}")


def bd_root() -> Path:
    return pyprojroot.find_root(pyprojroot.has_dir(".git"))
