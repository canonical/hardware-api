use crate::{ffi, IntoPy, PyAny, PyDowncastError, PyObject, PyTryFrom, Python, ToPyObject};

/// Represents the Python `None` object.
#[repr(transparent)]
pub struct PyNone(PyAny);

pyobject_native_type_named!(PyNone);
pyobject_native_type_extract!(PyNone);

impl PyNone {
    /// Returns the `None` object.
    #[inline]
    pub fn get(py: Python<'_>) -> &PyNone {
        unsafe { py.from_borrowed_ptr(ffi::Py_None()) }
    }
}

impl<'v> PyTryFrom<'v> for PyNone {
    fn try_from<V: Into<&'v PyAny>>(value: V) -> Result<&'v Self, crate::PyDowncastError<'v>> {
        let value: &PyAny = value.into();
        if value.is_none() {
            return unsafe { Ok(value.downcast_unchecked()) };
        }
        Err(PyDowncastError::new(value, "NoneType"))
    }

    fn try_from_exact<V: Into<&'v PyAny>>(
        value: V,
    ) -> Result<&'v Self, crate::PyDowncastError<'v>> {
        value.into().downcast()
    }

    unsafe fn try_from_unchecked<V: Into<&'v PyAny>>(value: V) -> &'v Self {
        let ptr = value.into() as *const _ as *const PyNone;
        &*ptr
    }
}

/// `()` is converted to Python `None`.
impl ToPyObject for () {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        PyNone::get(py).into()
    }
}

impl IntoPy<PyObject> for () {
    #[inline]
    fn into_py(self, py: Python<'_>) -> PyObject {
        PyNone::get(py).into()
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{PyDict, PyNone};
    use crate::{IntoPy, PyObject, Python, ToPyObject};

    #[test]
    fn test_none_is_none() {
        Python::with_gil(|py| {
            assert!(PyNone::get(py).downcast::<PyNone>().unwrap().is_none());
        })
    }

    #[test]
    fn test_unit_to_object_is_none() {
        Python::with_gil(|py| {
            assert!(().to_object(py).downcast::<PyNone>(py).is_ok());
        })
    }

    #[test]
    fn test_unit_into_py_is_none() {
        Python::with_gil(|py| {
            let obj: PyObject = ().into_py(py);
            assert!(obj.downcast::<PyNone>(py).is_ok());
        })
    }

    #[test]
    fn test_dict_is_not_none() {
        Python::with_gil(|py| {
            assert!(PyDict::new(py).downcast::<PyNone>().is_err());
        })
    }
}
