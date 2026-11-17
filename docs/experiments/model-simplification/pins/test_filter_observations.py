"""Fixed examples for validation precedence and Python boundary behavior."""
import pytest

from x_likes_mcp import tools
from x_likes_mcp.errors import ToolError


@pytest.mark.parametrize("year,start,end", [
    (None, None, None), (2026, None, None), (2026, "01", None),
    (2026, "01", "12"), (2026, "12", "01"),
    (2026, "01\n", "12\n"),
    (tools._YEAR_MIN, None, None), (tools._YEAR_MAX, None, None),
])
def test_valid_filter_observations(year, start, end):
    assert tools._validate_filter(year, start, end) is None


@pytest.mark.parametrize("year,start,end,reason", [
    (True, "bad", "bad", "year must be an integer"),
    (False, None, None, "year must be an integer"),
    (2026.0, None, None, "year must be an integer"),
    ("2026", None, None, "year must be an integer"),
    (None, "01", "02", "month_start requires year"),
    (2026, None, "02", "month_end requires month_start"),
    (None, None, "02", "month_end requires month_start"),
    (None, "00", "13", "month_start must match ^(0[1-9]|1[0-2])$"),
    (2026, "01", "13", "month_end must match ^(0[1-9]|1[0-2])$"),
    (2026, 1, None, "month_start must match ^(0[1-9]|1[0-2])$"),
    (2026, "01\n\n", None, "month_start must match ^(0[1-9]|1[0-2])$"),
])
def test_exact_first_error(year, start, end, reason):
    with pytest.raises(ToolError) as raised:
        tools._validate_filter(year, start, end)
    assert raised.value.category == "invalid_input"
    assert raised.value.message == f"invalid input for filter: {reason}"
    assert str(raised.value) == raised.value.message


@pytest.mark.parametrize("year", [tools._YEAR_MIN - 1, tools._YEAR_MAX + 1])
def test_year_range_error_precedes_month_errors(year):
    with pytest.raises(ToolError) as raised:
        tools._validate_filter(year, "bad", "bad")
    assert raised.value.message == (
        f"invalid input for filter: year must be in {tools._YEAR_MIN}..{tools._YEAR_MAX}"
    )
