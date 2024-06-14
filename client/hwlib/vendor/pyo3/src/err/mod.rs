use crate::panic::PanicException;
use crate::type_object::PyTypeInfo;
use crate::types::{PyTraceback, PyType};
use crate::{
    exceptions::{self, PyBaseException},
    ffi,
};
use crate::{IntoPy, Py, PyAny, PyObject, Python, ToPyObject};
use std::borrow::Cow;
use std::cell::UnsafeCell;
use std::ffi::CString;

mod err_state;
mod impls;

pub use err_state::PyErrArguments;
use err_state::{PyErrState, PyErrStateLazyFnOutput, PyErrStateNormalized};

/// Represents a Python exception.
///
/// To avoid needing access to [`Python`] in `Into` conversions to create `PyErr` (thus improving
/// compatibility with `?` and other Rust errors) this type supports creating exceptions instances
/// in a lazy fashion, where the full Python object for the exception is created only when needed.
///
/// Accessing the contained exception in any way, such as with [`value`](PyErr::value),
/// [`get_type`](PyErr::get_type), or [`is_instance`](PyErr::is_instance) will create the full
/// exception object if it was not already created.
pub struct PyErr {
    // Safety: can only hand out references when in the "normalized" state. Will never change
    // after normalization.
    //
    // The state is temporarily removed from the PyErr during normalization, to avoid
    // concurrent modifications.
    state: UnsafeCell<Option<PyErrState>>,
}

// The inner value is only accessed through ways that require proving the gil is held
#[cfg(feature = "nightly")]
unsafe impl crate::marker::Ungil for PyErr {}
unsafe impl Send for PyErr {}
unsafe impl Sync for PyErr {}

/// Represents the result of a Python call.
pub type PyResult<T> = Result<T, PyErr>;

/// Error that indicates a failure to convert a PyAny to a more specific Python type.
#[derive(Debug)]
pub struct PyDowncastError<'a> {
    from: &'a PyAny,
    to: Cow<'static, str>,
}

impl<'a> PyDowncastError<'a> {
    /// Create a new `PyDowncastError` representing a failure to convert the object
    /// `from` into the type named in `to`.
    pub fn new(from: &'a PyAny, to: impl Into<Cow<'static, str>>) -> Self {
        PyDowncastError {
            from,
            to: to.into(),
        }
    }
}

impl PyErr {
    /// Creates a new PyErr of type `T`.
    ///
    /// `args` can be:
    /// * a tuple: the exception instance will be created using the equivalent to the Python
    ///   expression `T(*tuple)`
    /// * any other value: the exception instance will be created using the equivalent to the Python
    ///   expression `T(value)`
    ///
    /// This exception instance will be initialized lazily. This avoids the need for the Python GIL
    /// to be held, but requires `args` to be `Send` and `Sync`. If `args` is not `Send` or `Sync`,
    /// consider using [`PyErr::from_value`] instead.
    ///
    /// If `T` does not inherit from `BaseException`, then a `TypeError` will be returned.
    ///
    /// If calling T's constructor with `args` raises an exception, that exception will be returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use pyo3::prelude::*;
    /// use pyo3::exceptions::PyTypeError;
    ///
    /// #[pyfunction]
    /// fn always_throws() -> PyResult<()> {
    ///     Err(PyErr::new::<PyTypeError, _>("Error message"))
    /// }
    /// #
    /// # Python::with_gil(|py| {
    /// #     let fun = pyo3::wrap_pyfunction!(always_throws, py).unwrap();
    /// #     let err = fun.call0().expect_err("called a function that should always return an error but the return value was Ok");
    /// #     assert!(err.is_instance_of::<PyTypeError>(py))
    /// # });
    /// ```
    ///
    /// In most cases, you can use a concrete exception's constructor instead:
    ///
    /// ```
    /// use pyo3::prelude::*;
    /// use pyo3::exceptions::PyTypeError;
    ///
    /// #[pyfunction]
    /// fn always_throws() -> PyResult<()> {
    ///     Err(PyTypeError::new_err("Error message"))
    /// }
    /// #
    /// # Python::with_gil(|py| {
    /// #     let fun = pyo3::wrap_pyfunction!(always_throws, py).unwrap();
    /// #     let err = fun.call0().expect_err("called a function that should always return an error but the return value was Ok");
    /// #     assert!(err.is_instance_of::<PyTypeError>(py))
    /// # });
    /// ```
    #[inline]
    pub fn new<T, A>(args: A) -> PyErr
    where
        T: PyTypeInfo,
        A: PyErrArguments + Send + Sync + 'static,
    {
        PyErr::from_state(PyErrState::Lazy(Box::new(move |py| {
            PyErrStateLazyFnOutput {
                ptype: T::type_object(py).into(),
                pvalue: args.arguments(py),
            }
        })))
    }

    /// Constructs a new PyErr from the given Python type and arguments.
    ///
    /// `ty` is the exception type; usually one of the standard exceptions
    /// like `exceptions::PyRuntimeError`.
    ///
    /// `args` is either a tuple or a single value, with the same meaning as in [`PyErr::new`].
    ///
    /// If `ty` does not inherit from `BaseException`, then a `TypeError` will be returned.
    ///
    /// If calling `ty` with `args` raises an exception, that exception will be returned.
    pub fn from_type<A>(ty: &PyType, args: A) -> PyErr
    where
        A: PyErrArguments + Send + Sync + 'static,
    {
        PyErr::from_state(PyErrState::lazy(ty, args))
    }

    /// Creates a new PyErr.
    ///
    /// If `obj` is a Python exception object, the PyErr will contain that object.
    ///
    /// If `obj` is a Python exception type object, this is equivalent to `PyErr::from_type(obj, ())`.
    ///
    /// Otherwise, a `TypeError` is created.
    ///
    /// # Examples
    /// ```rust
    /// use pyo3::prelude::*;
    /// use pyo3::exceptions::PyTypeError;
    /// use pyo3::types::{PyType, PyString};
    ///
    /// Python::with_gil(|py| {
    ///     // Case #1: Exception object
    ///     let err = PyErr::from_value(PyTypeError::new_err("some type error").value(py));
    ///     assert_eq!(err.to_string(), "TypeError: some type error");
    ///
    ///     // Case #2: Exception type
    ///     let err = PyErr::from_value(PyType::new::<PyTypeError>(py));
    ///     assert_eq!(err.to_string(), "TypeError: ");
    ///
    ///     // Case #3: Invalid exception value
    ///     let err = PyErr::from_value(PyString::new(py, "foo").into());
    ///     assert_eq!(
    ///         err.to_string(),
    ///         "TypeError: exceptions must derive from BaseException"
    ///     );
    /// });
    /// ```
    pub fn from_value(obj: &PyAny) -> PyErr {
        let state = if let Ok(obj) = obj.downcast::<PyBaseException>() {
            PyErrState::normalized(obj)
        } else {
            // Assume obj is Type[Exception]; let later normalization handle if this
            // is not the case
            PyErrState::lazy(obj, obj.py().None())
        };

        PyErr::from_state(state)
    }

    /// Returns the type of this exception.
    ///
    /// # Examples
    /// ```rust
    /// use pyo3::{exceptions::PyTypeError, types::PyType, PyErr, Python};
    ///
    /// Python::with_gil(|py| {
    ///     let err: PyErr = PyTypeError::new_err(("some type error",));
    ///     assert!(err.get_type(py).is(PyType::new::<PyTypeError>(py)));
    /// });
    /// ```
    pub fn get_type<'py>(&'py self, py: Python<'py>) -> &'py PyType {
        self.normalized(py).ptype(py)
    }

    /// Returns the value of this exception.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pyo3::{exceptions::PyTypeError, PyErr, Python};
    ///
    /// Python::with_gil(|py| {
    ///     let err: PyErr = PyTypeError::new_err(("some type error",));
    ///     assert!(err.is_instance_of::<PyTypeError>(py));
    ///     assert_eq!(err.value(py).to_string(), "some type error");
    /// });
    /// ```
    pub fn value<'py>(&'py self, py: Python<'py>) -> &'py PyBaseException {
        self.normalized(py).pvalue.as_ref(py)
    }

    /// Consumes self to take ownership of the exception value contained in this error.
    pub fn into_value(self, py: Python<'_>) -> Py<PyBaseException> {
        // NB technically this causes one reference count increase and decrease in quick succession
        // on pvalue, but it's probably not worth optimizing this right now for the additional code
        // complexity.
        let normalized = self.normalized(py);
        let exc = normalized.pvalue.clone_ref(py);
        if let Some(tb) = normalized.ptraceback(py) {
            unsafe {
                ffi::PyException_SetTraceback(exc.as_ptr(), tb.as_ptr());
            }
        }
        exc
    }

    /// Returns the traceback of this exception object.
    ///
    /// # Examples
    /// ```rust
    /// use pyo3::{exceptions::PyTypeError, Python};
    ///
    /// Python::with_gil(|py| {
    ///     let err = PyTypeError::new_err(("some type error",));
    ///     assert!(err.traceback(py).is_none());
    /// });
    /// ```
    pub fn traceback<'py>(&'py self, py: Python<'py>) -> Option<&'py PyTraceback> {
        self.normalized(py).ptraceback(py)
    }

    /// Gets whether an error is present in the Python interpreter's global state.
    #[inline]
    pub fn occurred(_: Python<'_>) -> bool {
        unsafe { !ffi::PyErr_Occurred().is_null() }
    }

    /// Takes the current error from the Python interpreter's global state and clears the global
    /// state. If no error is set, returns `None`.
    ///
    /// If the error is a `PanicException` (which would have originated from a panic in a pyo3
    /// callback) then this function will resume the panic.
    ///
    /// Use this function when it is not known if an error should be present. If the error is
    /// expected to have been set, for example from [`PyErr::occurred`] or by an error return value
    /// from a C FFI function, use [`PyErr::fetch`].
    pub fn take(py: Python<'_>) -> Option<PyErr> {
        Self::_take(py)
    }

    #[cfg(not(Py_3_12))]
    fn _take(py: Python<'_>) -> Option<PyErr> {
        let (ptype, pvalue, ptraceback) = unsafe {
            let mut ptype: *mut ffi::PyObject = std::ptr::null_mut();
            let mut pvalue: *mut ffi::PyObject = std::ptr::null_mut();
            let mut ptraceback: *mut ffi::PyObject = std::ptr::null_mut();
            ffi::PyErr_Fetch(&mut ptype, &mut pvalue, &mut ptraceback);

            // Convert to Py immediately so that any references are freed by early return.
            let ptype = PyObject::from_owned_ptr_or_opt(py, ptype);
            let pvalue = PyObject::from_owned_ptr_or_opt(py, pvalue);
            let ptraceback = PyObject::from_owned_ptr_or_opt(py, ptraceback);

            // A valid exception state should always have a non-null ptype, but the other two may be
            // null.
            let ptype = match ptype {
                Some(ptype) => ptype,
                None => {
                    debug_assert!(
                        pvalue.is_none(),
                        "Exception type was null but value was not null"
                    );
                    debug_assert!(
                        ptraceback.is_none(),
                        "Exception type was null but traceback was not null"
                    );
                    return None;
                }
            };

            (ptype, pvalue, ptraceback)
        };

        if ptype.as_ptr() == PanicException::type_object_raw(py).cast() {
            let msg = pvalue
                .as_ref()
                .and_then(|obj| obj.as_ref(py).str().ok())
                .map(|py_str| py_str.to_string_lossy().into())
                .unwrap_or_else(|| String::from("Unwrapped panic from Python code"));

            let state = PyErrState::FfiTuple {
                ptype,
                pvalue,
                ptraceback,
            };
            Self::print_panic_and_unwind(py, state, msg)
        }

        Some(PyErr::from_state(PyErrState::FfiTuple {
            ptype,
            pvalue,
            ptraceback,
        }))
    }

    #[cfg(Py_3_12)]
    fn _take(py: Python<'_>) -> Option<PyErr> {
        let state = PyErrStateNormalized::take(py)?;
        let pvalue = state.pvalue.as_ref(py);
        if pvalue.get_type().as_ptr() == PanicException::type_object_raw(py).cast() {
            let msg: String = pvalue
                .str()
                .map(|py_str| py_str.to_string_lossy().into())
                .unwrap_or_else(|_| String::from("Unwrapped panic from Python code"));
            Self::print_panic_and_unwind(py, PyErrState::Normalized(state), msg)
        }

        Some(PyErr::from_state(PyErrState::Normalized(state)))
    }

    fn print_panic_and_unwind(py: Python<'_>, state: PyErrState, msg: String) -> ! {
        eprintln!("--- PyO3 is resuming a panic after fetching a PanicException from Python. ---");
        eprintln!("Python stack trace below:");

        state.restore(py);

        unsafe {
            ffi::PyErr_PrintEx(0);
        }

        std::panic::resume_unwind(Box::new(msg))
    }

    /// Equivalent to [PyErr::take], but when no error is set:
    ///  - Panics in debug mode.
    ///  - Returns a `SystemError` in release mode.
    ///
    /// This behavior is consistent with Python's internal handling of what happens when a C return
    /// value indicates an error occurred but the global error state is empty. (A lack of exception
    /// should be treated as a bug in the code which returned an error code but did not set an
    /// exception.)
    ///
    /// Use this function when the error is expected to have been set, for example from
    /// [PyErr::occurred] or by an error return value from a C FFI function.
    #[cfg_attr(all(debug_assertions, track_caller), track_caller)]
    #[inline]
    pub fn fetch(py: Python<'_>) -> PyErr {
        const FAILED_TO_FETCH: &str = "attempted to fetch exception but none was set";
        match PyErr::take(py) {
            Some(err) => err,
            #[cfg(debug_assertions)]
            None => panic!("{}", FAILED_TO_FETCH),
            #[cfg(not(debug_assertions))]
            None => exceptions::PySystemError::new_err(FAILED_TO_FETCH),
        }
    }

    /// Creates a new exception type with the given name and docstring.
    ///
    /// - `base` can be an existing exception type to subclass, or a tuple of classes.
    /// - `dict` specifies an optional dictionary of class variables and methods.
    /// - `doc` will be the docstring seen by python users.
    ///
    ///
    /// # Errors
    ///
    /// This function returns an error if `name` is not of the form `<module>.<ExceptionName>`.
    ///
    /// # Panics
    ///
    /// This function will panic if  `name` or `doc` cannot be converted to [`CString`]s.
    pub fn new_type(
        py: Python<'_>,
        name: &str,
        doc: Option<&str>,
        base: Option<&PyType>,
        dict: Option<PyObject>,
    ) -> PyResult<Py<PyType>> {
        let base: *mut ffi::PyObject = match base {
            None => std::ptr::null_mut(),
            Some(obj) => obj.as_ptr(),
        };

        let dict: *mut ffi::PyObject = match dict {
            None => std::ptr::null_mut(),
            Some(obj) => obj.as_ptr(),
        };

        let null_terminated_name =
            CString::new(name).expect("Failed to initialize nul terminated exception name");

        let null_terminated_doc =
            doc.map(|d| CString::new(d).expect("Failed to initialize nul terminated docstring"));

        let null_terminated_doc_ptr = match null_terminated_doc.as_ref() {
            Some(c) => c.as_ptr(),
            None => std::ptr::null(),
        };

        let ptr = unsafe {
            ffi::PyErr_NewExceptionWithDoc(
                null_terminated_name.as_ptr(),
                null_terminated_doc_ptr,
                base,
                dict,
            )
        };

        unsafe { Py::from_owned_ptr_or_err(py, ptr) }
    }

    /// Prints a standard traceback to `sys.stderr`.
    pub fn display(&self, py: Python<'_>) {
        #[cfg(Py_3_12)]
        unsafe {
            ffi::PyErr_DisplayException(self.value(py).as_ptr())
        }

        #[cfg(not(Py_3_12))]
        unsafe {
            ffi::PyErr_Display(
                self.get_type(py).as_ptr(),
                self.value(py).as_ptr(),
                self.traceback(py)
                    .map_or(std::ptr::null_mut(), |traceback| traceback.as_ptr()),
            )
        }
    }

    /// Calls `sys.excepthook` and then prints a standard traceback to `sys.stderr`.
    pub fn print(&self, py: Python<'_>) {
        self.clone_ref(py).restore(py);
        unsafe { ffi::PyErr_PrintEx(0) }
    }

    /// Calls `sys.excepthook` and then prints a standard traceback to `sys.stderr`.
    ///
    /// Additionally sets `sys.last_{type,value,traceback,exc}` attributes to this exception.
    pub fn print_and_set_sys_last_vars(&self, py: Python<'_>) {
        self.clone_ref(py).restore(py);
        unsafe { ffi::PyErr_PrintEx(1) }
    }

    /// Returns true if the current exception matches the exception in `exc`.
    ///
    /// If `exc` is a class object, this also returns `true` when `self` is an instance of a subclass.
    /// If `exc` is a tuple, all exceptions in the tuple (and recursively in subtuples) are searched for a match.
    pub fn matches<T>(&self, py: Python<'_>, exc: T) -> bool
    where
        T: ToPyObject,
    {
        self.is_instance(py, exc.to_object(py).as_ref(py))
    }

    /// Returns true if the current exception is instance of `T`.
    #[inline]
    pub fn is_instance(&self, py: Python<'_>, ty: &PyAny) -> bool {
        (unsafe { ffi::PyErr_GivenExceptionMatches(self.get_type(py).as_ptr(), ty.as_ptr()) }) != 0
    }

    /// Returns true if the current exception is instance of `T`.
    #[inline]
    pub fn is_instance_of<T>(&self, py: Python<'_>) -> bool
    where
        T: PyTypeInfo,
    {
        self.is_instance(py, T::type_object(py))
    }

    /// Writes the error back to the Python interpreter's global state.
    /// This is the opposite of `PyErr::fetch()`.
    #[inline]
    pub fn restore(self, py: Python<'_>) {
        self.state
            .into_inner()
            .expect("PyErr state should never be invalid outside of normalization")
            .restore(py)
    }

    /// Reports the error as unraisable.
    ///
    /// This calls `sys.unraisablehook()` using the current exception and obj argument.
    ///
    /// This method is useful to report errors in situations where there is no good mechanism
    /// to report back to the Python land.  In Python this is used to indicate errors in
    /// background threads or destructors which are protected.  In Rust code this is commonly
    /// useful when you are calling into a Python callback which might fail, but there is no
    /// obvious way to handle this error other than logging it.
    ///
    /// Calling this method has the benefit that the error goes back into a standardized callback
    /// in Python which for instance allows unittests to ensure that no unraisable error
    /// actually happend by hooking `sys.unraisablehook`.
    ///
    /// Example:
    /// ```rust
    /// # use pyo3::prelude::*;
    /// # use pyo3::exceptions::PyRuntimeError;
    /// # fn failing_function() -> PyResult<()> { Err(PyRuntimeError::new_err("foo")) }
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| {
    ///     match failing_function() {
    ///         Err(pyerr) => pyerr.write_unraisable(py, None),
    ///         Ok(..) => { /* do something here */ }
    ///     }
    ///     Ok(())
    /// })
    /// # }
    #[inline]
    pub fn write_unraisable(self, py: Python<'_>, obj: Option<&PyAny>) {
        self.restore(py);
        unsafe { ffi::PyErr_WriteUnraisable(obj.map_or(std::ptr::null_mut(), |x| x.as_ptr())) }
    }

    /// Issues a warning message.
    ///
    /// May return an `Err(PyErr)` if warnings-as-errors is enabled.
    ///
    /// Equivalent to `warnings.warn()` in Python.
    ///
    /// The `category` should be one of the `Warning` classes available in
    /// [`pyo3::exceptions`](crate::exceptions), or a subclass.  The Python
    /// object can be retrieved using [`Python::get_type()`].
    ///
    /// Example:
    /// ```rust
    /// # use pyo3::prelude::*;
    /// # fn main() -> PyResult<()> {
    /// Python::with_gil(|py| {
    ///     let user_warning = py.get_type::<pyo3::exceptions::PyUserWarning>();
    ///     PyErr::warn(py, user_warning, "I am warning you", 0)?;
    ///     Ok(())
    /// })
    /// # }
    /// ```
    pub fn warn(py: Python<'_>, category: &PyAny, message: &str, stacklevel: i32) -> PyResult<()> {
        let message = CString::new(message)?;
        error_on_minusone(py, unsafe {
            ffi::PyErr_WarnEx(
                category.as_ptr(),
                message.as_ptr(),
                stacklevel as ffi::Py_ssize_t,
            )
        })
    }

    /// Issues a warning message, with more control over the warning attributes.
    ///
    /// May return a `PyErr` if warnings-as-errors is enabled.
    ///
    /// Equivalent to `warnings.warn_explicit()` in Python.
    ///
    /// The `category` should be one of the `Warning` classes available in
    /// [`pyo3::exceptions`](crate::exceptions), or a subclass.
    pub fn warn_explicit(
        py: Python<'_>,
        category: &PyAny,
        message: &str,
        filename: &str,
        lineno: i32,
        module: Option<&str>,
        registry: Option<&PyAny>,
    ) -> PyResult<()> {
        let message = CString::new(message)?;
        let filename = CString::new(filename)?;
        let module = module.map(CString::new).transpose()?;
        let module_ptr = match module {
            None => std::ptr::null_mut(),
            Some(s) => s.as_ptr(),
        };
        let registry: *mut ffi::PyObject = match registry {
            None => std::ptr::null_mut(),
            Some(obj) => obj.as_ptr(),
        };
        error_on_minusone(py, unsafe {
            ffi::PyErr_WarnExplicit(
                category.as_ptr(),
                message.as_ptr(),
                filename.as_ptr(),
                lineno,
                module_ptr,
                registry,
            )
        })
    }

    /// Clone the PyErr. This requires the GIL, which is why PyErr does not implement Clone.
    ///
    /// # Examples
    /// ```rust
    /// use pyo3::{exceptions::PyTypeError, PyErr, Python};
    /// Python::with_gil(|py| {
    ///     let err: PyErr = PyTypeError::new_err(("some type error",));
    ///     let err_clone = err.clone_ref(py);
    ///     assert!(err.get_type(py).is(err_clone.get_type(py)));
    ///     assert!(err.value(py).is(err_clone.value(py)));
    ///     match err.traceback(py) {
    ///         None => assert!(err_clone.traceback(py).is_none()),
    ///         Some(tb) => assert!(err_clone.traceback(py).unwrap().is(tb)),
    ///     }
    /// });
    /// ```
    #[inline]
    pub fn clone_ref(&self, py: Python<'_>) -> PyErr {
        PyErr::from_state(PyErrState::Normalized(self.normalized(py).clone()))
    }

    /// Return the cause (either an exception instance, or None, set by `raise ... from ...`)
    /// associated with the exception, as accessible from Python through `__cause__`.
    pub fn cause(&self, py: Python<'_>) -> Option<PyErr> {
        let value = self.value(py);
        let obj =
            unsafe { py.from_owned_ptr_or_opt::<PyAny>(ffi::PyException_GetCause(value.as_ptr())) };
        obj.map(Self::from_value)
    }

    /// Set the cause associated with the exception, pass `None` to clear it.
    pub fn set_cause(&self, py: Python<'_>, cause: Option<Self>) {
        let value = self.value(py);
        let cause = cause.map(|err| err.into_value(py));
        unsafe {
            // PyException_SetCause _steals_ a reference to cause, so must use .into_ptr()
            ffi::PyException_SetCause(
                value.as_ptr(),
                cause.map_or(std::ptr::null_mut(), Py::into_ptr),
            );
        }
    }

    #[inline]
    fn from_state(state: PyErrState) -> PyErr {
        PyErr {
            state: UnsafeCell::new(Some(state)),
        }
    }

    #[inline]
    fn normalized(&self, py: Python<'_>) -> &PyErrStateNormalized {
        if let Some(PyErrState::Normalized(n)) = unsafe {
            // Safety: self.state will never be written again once normalized.
            &*self.state.get()
        } {
            return n;
        }

        self.make_normalized(py)
    }

    #[cold]
    fn make_normalized(&self, py: Python<'_>) -> &PyErrStateNormalized {
        // This process is safe because:
        // - Access is guaranteed not to be concurrent thanks to `Python` GIL token
        // - Write happens only once, and then never will change again.
        // - State is set to None during the normalization process, so that a second
        //   concurrent normalization attempt will panic before changing anything.

        let state = unsafe {
            (*self.state.get())
                .take()
                .expect("Cannot normalize a PyErr while already normalizing it.")
        };

        unsafe {
            let self_state = &mut *self.state.get();
            *self_state = Some(PyErrState::Normalized(state.normalize(py)));
            match self_state {
                Some(PyErrState::Normalized(n)) => n,
                _ => unreachable!(),
            }
        }
    }
}

impl std::fmt::Debug for PyErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        Python::with_gil(|py| {
            f.debug_struct("PyErr")
                .field("type", self.get_type(py))
                .field("value", self.value(py))
                .field("traceback", &self.traceback(py))
                .finish()
        })
    }
}

impl std::fmt::Display for PyErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Python::with_gil(|py| {
            let value = self.value(py);
            let type_name = value.get_type().name().map_err(|_| std::fmt::Error)?;
            write!(f, "{}", type_name)?;
            if let Ok(s) = value.str() {
                write!(f, ": {}", &s.to_string_lossy())
            } else {
                write!(f, ": <exception str() failed>")
            }
        })
    }
}

impl std::error::Error for PyErr {}

impl IntoPy<PyObject> for PyErr {
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.into_value(py).into()
    }
}

impl ToPyObject for PyErr {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        self.clone_ref(py).into_py(py)
    }
}

impl<'a> IntoPy<PyObject> for &'a PyErr {
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.clone_ref(py).into_py(py)
    }
}

struct PyDowncastErrorArguments {
    from: Py<PyType>,
    to: Cow<'static, str>,
}

impl PyErrArguments for PyDowncastErrorArguments {
    fn arguments(self, py: Python<'_>) -> PyObject {
        format!(
            "'{}' object cannot be converted to '{}'",
            self.from
                .as_ref(py)
                .name()
                .unwrap_or("<failed to extract type name>"),
            self.to
        )
        .to_object(py)
    }
}

/// Convert `PyDowncastError` to Python `TypeError`.
impl<'a> std::convert::From<PyDowncastError<'a>> for PyErr {
    fn from(err: PyDowncastError<'_>) -> PyErr {
        let args = PyDowncastErrorArguments {
            from: err.from.get_type().into(),
            to: err.to,
        };

        exceptions::PyTypeError::new_err(args)
    }
}

impl<'a> std::error::Error for PyDowncastError<'a> {}

impl<'a> std::fmt::Display for PyDowncastError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "'{}' object cannot be converted to '{}'",
            self.from.get_type().name().map_err(|_| std::fmt::Error)?,
            self.to
        )
    }
}

pub fn panic_after_error(_py: Python<'_>) -> ! {
    unsafe {
        ffi::PyErr_Print();
    }
    panic!("Python API call failed");
}

/// Returns Ok if the error code is not -1.
#[inline]
pub(crate) fn error_on_minusone<T: SignedInteger>(py: Python<'_>, result: T) -> PyResult<()> {
    if result != T::MINUS_ONE {
        Ok(())
    } else {
        Err(PyErr::fetch(py))
    }
}

pub(crate) trait SignedInteger: Eq {
    const MINUS_ONE: Self;
}

macro_rules! impl_signed_integer {
    ($t:ty) => {
        impl SignedInteger for $t {
            const MINUS_ONE: Self = -1;
        }
    };
}

impl_signed_integer!(i8);
impl_signed_integer!(i16);
impl_signed_integer!(i32);
impl_signed_integer!(i64);
impl_signed_integer!(i128);
impl_signed_integer!(isize);

#[cfg(test)]
mod tests {
    use super::PyErrState;
    use crate::exceptions::{self, PyTypeError, PyValueError};
    use crate::tests::common::CatchWarnings;
    use crate::{PyErr, PyTypeInfo, Python};

    #[test]
    fn no_error() {
        assert!(Python::with_gil(PyErr::take).is_none());
    }

    #[test]
    fn set_valueerror() {
        Python::with_gil(|py| {
            let err: PyErr = exceptions::PyValueError::new_err("some exception message");
            assert!(err.is_instance_of::<exceptions::PyValueError>(py));
            err.restore(py);
            assert!(PyErr::occurred(py));
            let err = PyErr::fetch(py);
            assert!(err.is_instance_of::<exceptions::PyValueError>(py));
            assert_eq!(err.to_string(), "ValueError: some exception message");
        })
    }

    #[test]
    fn invalid_error_type() {
        Python::with_gil(|py| {
            let err: PyErr = PyErr::new::<crate::types::PyString, _>(());
            assert!(err.is_instance_of::<exceptions::PyTypeError>(py));
            err.restore(py);
            let err = PyErr::fetch(py);

            assert!(err.is_instance_of::<exceptions::PyTypeError>(py));
            assert_eq!(
                err.to_string(),
                "TypeError: exceptions must derive from BaseException"
            );
        })
    }

    #[test]
    fn set_typeerror() {
        Python::with_gil(|py| {
            let err: PyErr = exceptions::PyTypeError::new_err(());
            err.restore(py);
            assert!(PyErr::occurred(py));
            drop(PyErr::fetch(py));
        });
    }

    #[test]
    #[should_panic(expected = "new panic")]
    fn fetching_panic_exception_resumes_unwind() {
        use crate::panic::PanicException;

        Python::with_gil(|py| {
            let err: PyErr = PanicException::new_err("new panic");
            err.restore(py);
            assert!(PyErr::occurred(py));

            // should resume unwind
            let _ = PyErr::fetch(py);
        });
    }

    #[test]
    #[should_panic(expected = "new panic")]
    #[cfg(not(Py_3_12))]
    fn fetching_normalized_panic_exception_resumes_unwind() {
        use crate::panic::PanicException;

        Python::with_gil(|py| {
            let err: PyErr = PanicException::new_err("new panic");
            // Restoring an error doesn't normalize it before Python 3.12,
            // so we have to explicitly test this case.
            let _ = err.normalized(py);
            err.restore(py);
            assert!(PyErr::occurred(py));

            // should resume unwind
            let _ = PyErr::fetch(py);
        });
    }

    #[test]
    fn err_debug() {
        // Debug representation should be like the following (without the newlines):
        // PyErr {
        //     type: <class 'Exception'>,
        //     value: Exception('banana'),
        //     traceback: Some(<traceback object at 0x..)"
        // }

        Python::with_gil(|py| {
            let err = py
                .run("raise Exception('banana')", None, None)
                .expect_err("raising should have given us an error");

            let debug_str = format!("{:?}", err);
            assert!(debug_str.starts_with("PyErr { "));
            assert!(debug_str.ends_with(" }"));

            // strip "PyErr { " and " }"
            let mut fields = debug_str["PyErr { ".len()..debug_str.len() - 2].split(", ");

            assert_eq!(fields.next().unwrap(), "type: <class 'Exception'>");
            assert_eq!(fields.next().unwrap(), "value: Exception('banana')");

            let traceback = fields.next().unwrap();
            assert!(traceback.starts_with("traceback: Some(<traceback object at 0x"));
            assert!(traceback.ends_with(">)"));

            assert!(fields.next().is_none());
        });
    }

    #[test]
    fn err_display() {
        Python::with_gil(|py| {
            let err = py
                .run("raise Exception('banana')", None, None)
                .expect_err("raising should have given us an error");
            assert_eq!(err.to_string(), "Exception: banana");
        });
    }

    #[test]
    fn test_pyerr_send_sync() {
        fn is_send<T: Send>() {}
        fn is_sync<T: Sync>() {}

        is_send::<PyErr>();
        is_sync::<PyErr>();

        is_send::<PyErrState>();
        is_sync::<PyErrState>();
    }

    #[test]
    fn test_pyerr_matches() {
        Python::with_gil(|py| {
            let err = PyErr::new::<PyValueError, _>("foo");
            assert!(err.matches(py, PyValueError::type_object(py)));

            assert!(err.matches(
                py,
                (PyValueError::type_object(py), PyTypeError::type_object(py))
            ));

            assert!(!err.matches(py, PyTypeError::type_object(py)));

            // String is not a valid exception class, so we should get a TypeError
            let err: PyErr = PyErr::from_type(crate::types::PyString::type_object(py), "foo");
            assert!(err.matches(py, PyTypeError::type_object(py)));
        })
    }

    #[test]
    fn test_pyerr_cause() {
        Python::with_gil(|py| {
            let err = py
                .run("raise Exception('banana')", None, None)
                .expect_err("raising should have given us an error");
            assert!(err.cause(py).is_none());

            let err = py
                .run(
                    "raise Exception('banana') from Exception('apple')",
                    None,
                    None,
                )
                .expect_err("raising should have given us an error");
            let cause = err
                .cause(py)
                .expect("raising from should have given us a cause");
            assert_eq!(cause.to_string(), "Exception: apple");

            err.set_cause(py, None);
            assert!(err.cause(py).is_none());

            let new_cause = exceptions::PyValueError::new_err("orange");
            err.set_cause(py, Some(new_cause));
            let cause = err
                .cause(py)
                .expect("set_cause should have given us a cause");
            assert_eq!(cause.to_string(), "ValueError: orange");
        });
    }

    #[test]
    fn warnings() {
        // Note: although the warning filter is interpreter global, keeping the
        // GIL locked should prevent effects to be visible to other testing
        // threads.
        Python::with_gil(|py| {
            let cls = py.get_type::<exceptions::PyUserWarning>();

            // Reset warning filter to default state
            let warnings = py.import("warnings").unwrap();
            warnings.call_method0("resetwarnings").unwrap();

            // First, test the warning is emitted
            assert_warnings!(
                py,
                { PyErr::warn(py, cls, "I am warning you", 0).unwrap() },
                [(exceptions::PyUserWarning, "I am warning you")]
            );

            // Test with raising
            warnings
                .call_method1("simplefilter", ("error", cls))
                .unwrap();
            PyErr::warn(py, cls, "I am warning you", 0).unwrap_err();

            // Test with error for an explicit module
            warnings.call_method0("resetwarnings").unwrap();
            warnings
                .call_method1("filterwarnings", ("error", "", cls, "pyo3test"))
                .unwrap();

            // This has the wrong module and will not raise, just be emitted
            assert_warnings!(
                py,
                { PyErr::warn(py, cls, "I am warning you", 0).unwrap() },
                [(exceptions::PyUserWarning, "I am warning you")]
            );

            let err =
                PyErr::warn_explicit(py, cls, "I am warning you", "pyo3test.py", 427, None, None)
                    .unwrap_err();
            assert!(err
                .value(py)
                .getattr("args")
                .unwrap()
                .get_item(0)
                .unwrap()
                .eq("I am warning you")
                .unwrap());

            // Finally, reset filter again
            warnings.call_method0("resetwarnings").unwrap();
        });
    }
}
