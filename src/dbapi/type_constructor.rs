use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDate, PyDateTime, PyTime};

#[pyfunction]
#[pyo3(name = "Date")]
pub fn date(py: Python, year: i32, month: u8, day: u8) -> PyResult<Bound<PyDate>> {
    PyDate::new(py, year, month, day)
}

#[pyfunction]
#[pyo3(name = "Time")]
pub fn time(py: Python, hour: u8, minute: u8, second: u8) -> PyResult<Bound<PyTime>> {
    PyTime::new(py, hour, minute, second, 0, None)
}

#[pyfunction]
#[pyo3(name = "Timestamp")]
pub fn timestamp(
    py: Python,
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
) -> PyResult<Bound<PyDateTime>> {
    PyDateTime::new(py, year, month, day, hour, minute, second, 0, None)
}

#[pyfunction]
#[pyo3(name = "DateFromTicks")]
pub fn date_from_ticks(py: Python, ticks: i64) -> PyResult<Bound<PyDate>> {
    // This cast is lossless for all timestamps representable by PyDate: f64 represents every
    // integer exactly through 2^53, while PyDate's timestamp range is within 2^38.
    PyDate::from_timestamp(py, ticks as f64)
}

#[pyfunction]
#[pyo3(name = "TimeFromTicks")]
pub fn time_from_ticks(py: Python, ticks: i64) -> PyResult<Bound<PyTime>> {
    let (hour, minute, second) = utc_time_of_day(ticks).ok_or_else(|| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid timestamp: {}", ticks))
    })?;
    PyTime::new(py, hour, minute, second, 0, None)
}

/// Splits `ticks` (seconds since the Unix epoch) into the UTC hour, minute and second.
///
/// Returns `None` outside the years -262143..=262142. That range is inherited from the chrono
/// implementation this replaced, so the same ticks keep raising `ValueError`.
fn utc_time_of_day(ticks: i64) -> Option<(u8, u8, u8)> {
    const MIN_TICKS: i64 = -8_334_601_228_800;
    const MAX_TICKS: i64 = 8_210_266_876_799;
    if !(MIN_TICKS..=MAX_TICKS).contains(&ticks) {
        return None;
    }
    let second_of_day = ticks.rem_euclid(86_400);
    Some((
        (second_of_day / 3600) as u8,
        (second_of_day / 60 % 60) as u8,
        (second_of_day % 60) as u8,
    ))
}

#[pyfunction]
#[pyo3(name = "TimestampFromTicks")]
pub fn timestamp_from_ticks(py: Python, ticks: f64) -> PyResult<Bound<PyDateTime>> {
    PyDateTime::from_timestamp(py, ticks, None)
}

#[pyfunction]
#[pyo3(name = "Binary")]
pub fn binary(py: Python, x: &Bound<'_, PyAny>) -> PyResult<Py<PyBytes>> {
    // Mimics Python's bytes(x) behavior
    let bytes_type = py.get_type::<PyBytes>();
    let result = bytes_type.call1((x,))?;
    Ok(result.extract::<Py<PyBytes>>()?)
}

#[cfg(test)]
mod tests {
    use super::utc_time_of_day;

    #[test]
    fn utc_time_of_day_splits_ticks() {
        assert_eq!(utc_time_of_day(0), Some((0, 0, 0)));
        assert_eq!(utc_time_of_day(86_399), Some((23, 59, 59)));
        assert_eq!(utc_time_of_day(86_400), Some((0, 0, 0)));
        assert_eq!(utc_time_of_day(1_700_000_000), Some((22, 13, 20)));
    }

    #[test]
    fn utc_time_of_day_wraps_backwards_before_the_epoch() {
        assert_eq!(utc_time_of_day(-1), Some((23, 59, 59)));
        assert_eq!(utc_time_of_day(-86_401), Some((23, 59, 59)));
    }

    #[test]
    fn utc_time_of_day_rejects_ticks_outside_the_supported_years() {
        assert_eq!(utc_time_of_day(-8_334_601_228_800), Some((0, 0, 0)));
        assert_eq!(utc_time_of_day(8_210_266_876_799), Some((23, 59, 59)));
        assert_eq!(utc_time_of_day(-8_334_601_228_801), None);
        assert_eq!(utc_time_of_day(8_210_266_876_800), None);
        assert_eq!(utc_time_of_day(i64::MIN), None);
        assert_eq!(utc_time_of_day(i64::MAX), None);
    }
}
