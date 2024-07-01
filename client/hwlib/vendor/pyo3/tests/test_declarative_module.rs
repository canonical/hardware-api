#![cfg(feature = "experimental-declarative-modules")]

use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
#[cfg(not(Py_LIMITED_API))]
use pyo3::types::PyBool;

#[path = "../src/tests/common.rs"]
mod common;

mod some_module {
    use pyo3::prelude::*;

    #[pyclass]
    pub struct SomePyClass;
}

#[pyclass]
struct ValueClass {
    value: usize,
}

#[pymethods]
impl ValueClass {
    #[new]
    fn new(value: usize) -> Self {
        Self { value }
    }
}

#[pyclass(module = "module")]
struct LocatedClass {}

#[pyfunction]
fn double(x: usize) -> usize {
    x * 2
}

create_exception!(
    declarative_module,
    MyError,
    PyException,
    "Some description."
);

/// A module written using declarative syntax.
#[pymodule]
mod declarative_module {
    #[pymodule_export]
    use super::declarative_submodule;
    #[pymodule_export]
    // This is not a real constraint but to test cfg attribute support
    #[cfg(not(Py_LIMITED_API))]
    use super::LocatedClass;
    use super::*;
    #[pymodule_export]
    use super::{declarative_module2, double, MyError, ValueClass as Value};

    // test for #4036
    #[pymodule_export]
    use super::some_module::SomePyClass;

    #[pymodule]
    mod inner {
        use super::*;

        #[pyfunction]
        fn triple(x: usize) -> usize {
            x * 3
        }

        #[pyclass]
        struct Struct;

        #[pymethods]
        impl Struct {
            #[new]
            fn new() -> Self {
                Self
            }
        }

        #[pyclass]
        enum Enum {
            A,
            B,
        }
    }

    #[pymodule_init]
    fn init(m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add("double2", m.getattr("double")?)
    }
}

#[pyfunction]
fn double_value(v: &ValueClass) -> usize {
    v.value * 2
}

#[pymodule]
mod declarative_submodule {
    #[pymodule_export]
    use super::{double, double_value};
}

#[pymodule]
#[pyo3(name = "declarative_module_renamed")]
mod declarative_module2 {
    #[pymodule_export]
    use super::double;
}

#[test]
fn test_declarative_module() {
    Python::with_gil(|py| {
        let m = pyo3::wrap_pymodule!(declarative_module)(py).into_bound(py);
        py_assert!(
            py,
            m,
            "m.__doc__ == 'A module written using declarative syntax.'"
        );

        py_assert!(py, m, "m.double(2) == 4");
        py_assert!(py, m, "m.inner.triple(3) == 9");
        py_assert!(py, m, "m.declarative_submodule.double(4) == 8");
        py_assert!(
            py,
            m,
            "m.declarative_submodule.double_value(m.ValueClass(1)) == 2"
        );
        py_assert!(py, m, "str(m.MyError('foo')) == 'foo'");
        py_assert!(py, m, "m.declarative_module_renamed.double(2) == 4");
        #[cfg(Py_LIMITED_API)]
        py_assert!(py, m, "not hasattr(m, 'LocatedClass')");
        #[cfg(not(Py_LIMITED_API))]
        py_assert!(py, m, "hasattr(m, 'LocatedClass')");
        py_assert!(py, m, "isinstance(m.inner.Struct(), m.inner.Struct)");
        py_assert!(py, m, "isinstance(m.inner.Enum.A, m.inner.Enum)");
    })
}

#[cfg(not(Py_LIMITED_API))]
#[pyclass(extends = PyBool)]
struct ExtendsBool;

#[cfg(not(Py_LIMITED_API))]
#[pymodule]
mod class_initialization_module {
    #[pymodule_export]
    use super::ExtendsBool;
}

#[test]
#[cfg(not(Py_LIMITED_API))]
fn test_class_initialization_fails() {
    Python::with_gil(|py| {
        let err = class_initialization_module::_PYO3_DEF
            .make_module(py)
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            "RuntimeError: An error occurred while initializing class ExtendsBool"
        );
    })
}

#[pymodule]
mod r#type {
    #[pymodule_export]
    use super::double;
}

#[test]
fn test_raw_ident_module() {
    Python::with_gil(|py| {
        let m = pyo3::wrap_pymodule!(r#type)(py).into_bound(py);
        py_assert!(py, m, "m.double(2) == 4");
    })
}
