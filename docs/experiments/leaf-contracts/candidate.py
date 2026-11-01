"""Isolated replacement trial; not imported by the Lesscode runtime."""
from typing import Annotated
from pydantic import AfterValidator, BaseModel, ConfigDict, Field, model_validator


def relative_path(value):
    if (not value.strip() or "\x00" in value or "\\" in value or ":" in value
            or any(p in ("", ".", "..") or p.endswith((" ", ".")) for p in value.split("/"))):
        raise ValueError("expected relative POSIX path")
    return value


def nonempty(value):
    if not value.strip() or "\x00" in value:
        raise ValueError("expected nonempty string without NUL")
    return value


PathName = Annotated[str, AfterValidator(relative_path)]
Name = Annotated[str, AfterValidator(nonempty)]
Positive = Annotated[int, Field(ge=1)]
Nonnegative = Annotated[int, Field(ge=0)]


class Record(BaseModel):
    model_config = ConfigDict(strict=True, extra="forbid", frozen=True)


class Limits(Record):
    timeout_seconds: Annotated[float, Field(gt=0, allow_inf_nan=False)]
    max_log_bytes: Positive


class FileComplexity(Record):
    path: PathName
    nloc: Nonnegative


class SourceSpan(Record):
    path: PathName
    start_line: Positive
    end_line: Positive

    @model_validator(mode="after")
    def ordered(self):
        if self.end_line < self.start_line:
            raise ValueError("end_line precedes start_line")
        return self


class FunctionComplexity(SourceSpan):
    name: Name
    long_name: Name
    ccn: Positive
    nloc: Nonnegative
    token_count: Nonnegative


class ClonePair(Record):
    first: SourceSpan
    second: SourceSpan
