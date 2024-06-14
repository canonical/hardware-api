#![cfg(feature = "smallvec")]

//!  Conversions to and from [smallvec](https://docs.rs/smallvec/).
//!
//! # Setup
//!
//! To use this feature, add this to your **`Cargo.toml`**:
//!
//! ```toml
//! [dependencies]
//! # change * to the latest versions
//! smallvec = "*"
#![doc = concat!("pyo3 = { version = \"", env!("CARGO_PKG_VERSION"),  "\", features = [\"smallvec\"] }")]
//! ```
//!
//! Note that you must use compatible versions of smallvec and PyO3.
//! The required smallvec version may vary based on the version of PyO3.
use crate::exceptions::PyTypeError;
#[cfg(feature = "experimental-inspect")]
use crate::inspect::types::TypeInfo;
use crate::types::list::new_from_iter;
use crate::types::{PySequence, PyString};
use crate::{
    ffi, FromPyObject, IntoPy, PyAny, PyDowncastError, PyObject, PyResult, Python, ToPyObject,
};
use smallvec::{Array, SmallVec};

impl<A> ToPyObject for SmallVec<A>
where
    A: Array,
    A::Item: ToPyObject,
{
    fn to_object(&self, py: Python<'_>) -> PyObject {
        self.as_slice().to_object(py)
    }
}

impl<A> IntoPy<PyObject> for SmallVec<A>
where
    A: Array,
    A::Item: IntoPy<PyObject>,
{
    fn into_py(self, py: Python<'_>) -> PyObject {
        let mut iter = self.into_iter().map(|e| e.into_py(py));
        let list = new_from_iter(py, &mut iter);
        list.into()
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        TypeInfo::list_of(A::Item::type_output())
    }
}

impl<'a, A> FromPyObject<'a> for SmallVec<A>
where
    A: Array,
    A::Item: FromPyObject<'a>,
{
    fn extract(obj: &'a PyAny) -> PyResult<Self> {
        if obj.is_instance_of::<PyString>() {
            return Err(PyTypeError::new_err("Can't extract `str` to `SmallVec`"));
        }
        extract_sequence(obj)
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_input() -> TypeInfo {
        TypeInfo::sequence_of(A::Item::type_input())
    }
}

fn extract_sequence<'s, A>(obj: &'s PyAny) -> PyResult<SmallVec<A>>
where
    A: Array,
    A::Item: FromPyObject<'s>,
{
    // Types that pass `PySequence_Check` usually implement enough of the sequence protocol
    // to support this function and if not, we will only fail extraction safely.
    let seq: &PySequence = unsafe {
        if ffi::PySequence_Check(obj.as_ptr()) != 0 {
            obj.downcast_unchecked()
        } else {
            return Err(PyDowncastError::new(obj, "Sequence").into());
        }
    };

    let mut sv = SmallVec::with_capacity(seq.len().unwrap_or(0));
    for item in seq.iter()? {
        sv.push(item?.extract::<A::Item>()?);
    }
    Ok(sv)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{PyDict, PyList};

    #[test]
    fn test_smallvec_into_py() {
        Python::with_gil(|py| {
            let sv: SmallVec<[u64; 8]> = [1, 2, 3, 4, 5].iter().cloned().collect();
            let hso: PyObject = sv.clone().into_py(py);
            let l = PyList::new(py, [1, 2, 3, 4, 5]);
            assert!(l.eq(hso).unwrap());
        });
    }

    #[test]
    fn test_smallvec_from_py_object() {
        Python::with_gil(|py| {
            let l = PyList::new(py, [1, 2, 3, 4, 5]);
            let sv: SmallVec<[u64; 8]> = l.extract().unwrap();
            assert_eq!(sv.as_slice(), [1, 2, 3, 4, 5]);
        });
    }

    #[test]
    fn test_smallvec_from_py_object_fails() {
        Python::with_gil(|py| {
            let dict = PyDict::new(py);
            let sv: PyResult<SmallVec<[u64; 8]>> = dict.extract();
            assert_eq!(
                sv.unwrap_err().to_string(),
                "TypeError: 'dict' object cannot be converted to 'Sequence'"
            );
        });
    }

    #[test]
    fn test_smallvec_to_object() {
        Python::with_gil(|py| {
            let sv: SmallVec<[u64; 8]> = [1, 2, 3, 4, 5].iter().cloned().collect();
            let hso: PyObject = sv.to_object(py);
            let l = PyList::new(py, [1, 2, 3, 4, 5]);
            assert!(l.eq(hso).unwrap());
        });
    }
}
