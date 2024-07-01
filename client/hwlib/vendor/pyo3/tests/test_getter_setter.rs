#![cfg(feature = "macros")]

use std::cell::Cell;

use pyo3::prelude::*;
use pyo3::py_run;
use pyo3::types::{IntoPyDict, PyList};

#[path = "../src/tests/common.rs"]
mod common;

#[pyclass]
struct ClassWithProperties {
    num: i32,
}

#[pymethods]
impl ClassWithProperties {
    fn get_num(&self) -> i32 {
        self.num
    }

    #[getter(DATA)]
    /// a getter for data
    fn get_data(&self) -> i32 {
        self.num
    }
    #[setter(DATA)]
    fn set_data(&mut self, value: i32) {
        self.num = value;
    }

    #[getter]
    /// a getter with a type un-wrapped by PyResult
    fn get_unwrapped(&self) -> i32 {
        self.num
    }

    #[setter]
    fn set_unwrapped(&mut self, value: i32) {
        self.num = value;
    }

    #[setter]
    fn set_from_len(&mut self, #[pyo3(from_py_with = "extract_len")] value: i32) {
        self.num = value;
    }

    #[setter]
    fn set_from_any(&mut self, value: &Bound<'_, PyAny>) -> PyResult<()> {
        self.num = value.extract()?;
        Ok(())
    }

    #[getter]
    fn get_data_list<'py>(&self, py: Python<'py>) -> Bound<'py, PyList> {
        PyList::new_bound(py, [self.num])
    }
}

fn extract_len(any: &Bound<'_, PyAny>) -> PyResult<i32> {
    any.len().map(|len| len as i32)
}

#[test]
fn class_with_properties() {
    Python::with_gil(|py| {
        let inst = Py::new(py, ClassWithProperties { num: 10 }).unwrap();

        py_run!(py, inst, "assert inst.get_num() == 10");
        py_run!(py, inst, "assert inst.get_num() == inst.DATA");
        py_run!(py, inst, "inst.DATA = 20");
        py_run!(py, inst, "assert inst.get_num() == 20 == inst.DATA");

        py_expect_exception!(py, inst, "del inst.DATA", PyAttributeError);

        py_run!(py, inst, "assert inst.get_num() == inst.unwrapped == 20");
        py_run!(py, inst, "inst.unwrapped = 42");
        py_run!(py, inst, "assert inst.get_num() == inst.unwrapped == 42");
        py_run!(py, inst, "assert inst.data_list == [42]");

        py_run!(py, inst, "inst.from_len = [0, 0, 0]");
        py_run!(py, inst, "assert inst.get_num() == 3");

        py_run!(py, inst, "inst.from_any = 15");
        py_run!(py, inst, "assert inst.get_num() == 15");

        let d = [("C", py.get_type_bound::<ClassWithProperties>())].into_py_dict_bound(py);
        py_assert!(py, *d, "C.DATA.__doc__ == 'a getter for data'");
    });
}

#[pyclass]
struct GetterSetter {
    #[pyo3(get, set)]
    num: i32,
    #[pyo3(get, set)]
    text: String,
}

#[pymethods]
impl GetterSetter {
    fn get_num2(&self) -> i32 {
        self.num
    }
}

#[test]
fn getter_setter_autogen() {
    Python::with_gil(|py| {
        let inst = Py::new(
            py,
            GetterSetter {
                num: 10,
                text: "Hello".to_string(),
            },
        )
        .unwrap();

        py_run!(py, inst, "assert inst.num == 10");
        py_run!(py, inst, "inst.num = 20; assert inst.num == 20");
        py_run!(
            py,
            inst,
            "assert inst.text == 'Hello'; inst.text = 'There'; assert inst.text == 'There'"
        );
    });
}

#[pyclass]
struct RefGetterSetter {
    num: i32,
}

#[pymethods]
impl RefGetterSetter {
    #[getter]
    fn get_num(slf: PyRef<'_, Self>) -> i32 {
        slf.num
    }

    #[setter]
    fn set_num(mut slf: PyRefMut<'_, Self>, value: i32) {
        slf.num = value;
    }
}

#[test]
fn ref_getter_setter() {
    // Regression test for #837
    Python::with_gil(|py| {
        let inst = Py::new(py, RefGetterSetter { num: 10 }).unwrap();

        py_run!(py, inst, "assert inst.num == 10");
        py_run!(py, inst, "inst.num = 20; assert inst.num == 20");
    });
}

#[pyclass]
struct TupleClassGetterSetter(i32);

#[pymethods]
impl TupleClassGetterSetter {
    #[getter(num)]
    fn get_num(&self) -> i32 {
        self.0
    }

    #[setter(num)]
    fn set_num(&mut self, value: i32) {
        self.0 = value;
    }
}

#[test]
fn tuple_struct_getter_setter() {
    Python::with_gil(|py| {
        let inst = Py::new(py, TupleClassGetterSetter(10)).unwrap();

        py_assert!(py, inst, "inst.num == 10");
        py_run!(py, inst, "inst.num = 20");
        py_assert!(py, inst, "inst.num == 20");
    });
}

#[pyclass(get_all, set_all)]
struct All {
    num: i32,
}

#[test]
fn get_set_all() {
    Python::with_gil(|py| {
        let inst = Py::new(py, All { num: 10 }).unwrap();

        py_run!(py, inst, "assert inst.num == 10");
        py_run!(py, inst, "inst.num = 20; assert inst.num == 20");
    });
}

#[pyclass(get_all)]
struct All2 {
    #[pyo3(set)]
    num: i32,
}

#[test]
fn get_all_and_set() {
    Python::with_gil(|py| {
        let inst = Py::new(py, All2 { num: 10 }).unwrap();

        py_run!(py, inst, "assert inst.num == 10");
        py_run!(py, inst, "inst.num = 20; assert inst.num == 20");
    });
}

#[pyclass]
struct CellGetterSetter {
    #[pyo3(get, set)]
    cell_inner: Cell<i32>,
}

#[test]
fn cell_getter_setter() {
    let c = CellGetterSetter {
        cell_inner: Cell::new(10),
    };
    Python::with_gil(|py| {
        let inst = Py::new(py, c).unwrap().to_object(py);
        let cell = Cell::new(20).to_object(py);

        py_run!(py, cell, "assert cell == 20");
        py_run!(py, inst, "assert inst.cell_inner == 10");
        py_run!(
            py,
            inst,
            "inst.cell_inner = 20; assert inst.cell_inner == 20"
        );
    });
}

#[test]
fn borrowed_value_with_lifetime_of_self() {
    #[pyclass]
    struct BorrowedValue {}

    #[pymethods]
    impl BorrowedValue {
        #[getter]
        fn value(&self) -> &str {
            "value"
        }
    }

    Python::with_gil(|py| {
        let inst = Py::new(py, BorrowedValue {}).unwrap().to_object(py);

        py_run!(py, inst, "assert inst.value == 'value'");
    });
}
