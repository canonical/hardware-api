use crate::{
    exceptions::{PyAttributeError, PyNotImplementedError, PyRuntimeError, PyValueError},
    ffi,
    impl_::freelist::FreeList,
    impl_::pycell::{GetBorrowChecker, PyClassMutability},
    internal_tricks::extract_c_string,
    pycell::PyCellLayout,
    pyclass_init::PyObjectInit,
    types::PyBool,
    Py, PyAny, PyCell, PyClass, PyErr, PyMethodDefType, PyNativeType, PyResult, PyTypeInfo, Python,
};
use std::{
    borrow::Cow,
    ffi::{CStr, CString},
    marker::PhantomData,
    os::raw::{c_int, c_void},
    ptr::NonNull,
    thread,
};

mod lazy_type_object;
pub use lazy_type_object::LazyTypeObject;

/// Gets the offset of the dictionary from the start of the object in bytes.
#[inline]
pub fn dict_offset<T: PyClass>() -> ffi::Py_ssize_t {
    PyCell::<T>::dict_offset()
}

/// Gets the offset of the weakref list from the start of the object in bytes.
#[inline]
pub fn weaklist_offset<T: PyClass>() -> ffi::Py_ssize_t {
    PyCell::<T>::weaklist_offset()
}

/// Represents the `__dict__` field for `#[pyclass]`.
pub trait PyClassDict {
    /// Initial form of a [PyObject](crate::ffi::PyObject) `__dict__` reference.
    const INIT: Self;
    /// Empties the dictionary of its key-value pairs.
    #[inline]
    fn clear_dict(&mut self, _py: Python<'_>) {}
    private_decl! {}
}

/// Represents the `__weakref__` field for `#[pyclass]`.
pub trait PyClassWeakRef {
    /// Initializes a `weakref` instance.
    const INIT: Self;
    /// Clears the weak references to the given object.
    ///
    /// # Safety
    /// - `_obj` must be a pointer to the pyclass instance which contains `self`.
    /// - The GIL must be held.
    #[inline]
    unsafe fn clear_weakrefs(&mut self, _obj: *mut ffi::PyObject, _py: Python<'_>) {}
    private_decl! {}
}

/// Zero-sized dummy field.
pub struct PyClassDummySlot;

impl PyClassDict for PyClassDummySlot {
    private_impl! {}
    const INIT: Self = PyClassDummySlot;
}

impl PyClassWeakRef for PyClassDummySlot {
    private_impl! {}
    const INIT: Self = PyClassDummySlot;
}

/// Actual dict field, which holds the pointer to `__dict__`.
///
/// `#[pyclass(dict)]` automatically adds this.
#[repr(transparent)]
pub struct PyClassDictSlot(*mut ffi::PyObject);

impl PyClassDict for PyClassDictSlot {
    private_impl! {}
    const INIT: Self = Self(std::ptr::null_mut());
    #[inline]
    fn clear_dict(&mut self, _py: Python<'_>) {
        if !self.0.is_null() {
            unsafe { ffi::PyDict_Clear(self.0) }
        }
    }
}

/// Actual weakref field, which holds the pointer to `__weakref__`.
///
/// `#[pyclass(weakref)]` automatically adds this.
#[repr(transparent)]
pub struct PyClassWeakRefSlot(*mut ffi::PyObject);

impl PyClassWeakRef for PyClassWeakRefSlot {
    private_impl! {}
    const INIT: Self = Self(std::ptr::null_mut());
    #[inline]
    unsafe fn clear_weakrefs(&mut self, obj: *mut ffi::PyObject, _py: Python<'_>) {
        if !self.0.is_null() {
            ffi::PyObject_ClearWeakRefs(obj)
        }
    }
}

/// This type is used as a "dummy" type on which dtolnay specializations are
/// applied to apply implementations from `#[pymethods]`
pub struct PyClassImplCollector<T>(PhantomData<T>);

impl<T> PyClassImplCollector<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T> Default for PyClassImplCollector<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Clone for PyClassImplCollector<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for PyClassImplCollector<T> {}

pub struct PyClassItems {
    pub methods: &'static [PyMethodDefType],
    pub slots: &'static [ffi::PyType_Slot],
}

// Allow PyClassItems in statics
unsafe impl Sync for PyClassItems {}

/// Implements the underlying functionality of `#[pyclass]`, assembled by various proc macros.
///
/// Users are discouraged from implementing this trait manually; it is a PyO3 implementation detail
/// and may be changed at any time.
pub trait PyClassImpl: Sized + 'static {
    /// #[pyclass(subclass)]
    const IS_BASETYPE: bool = false;

    /// #[pyclass(extends=...)]
    const IS_SUBCLASS: bool = false;

    /// #[pyclass(mapping)]
    const IS_MAPPING: bool = false;

    /// #[pyclass(sequence)]
    const IS_SEQUENCE: bool = false;

    /// Base class
    type BaseType: PyTypeInfo + PyClassBaseType;

    /// Immutable or mutable
    type PyClassMutability: PyClassMutability + GetBorrowChecker<Self>;

    /// Specify this class has `#[pyclass(dict)]` or not.
    type Dict: PyClassDict;

    /// Specify this class has `#[pyclass(weakref)]` or not.
    type WeakRef: PyClassWeakRef;

    /// The closest native ancestor. This is `PyAny` by default, and when you declare
    /// `#[pyclass(extends=PyDict)]`, it's `PyDict`.
    type BaseNativeType: PyTypeInfo + PyNativeType;

    /// This handles following two situations:
    /// 1. In case `T` is `Send`, stub `ThreadChecker` is used and does nothing.
    ///    This implementation is used by default. Compile fails if `T: !Send`.
    /// 2. In case `T` is `!Send`, `ThreadChecker` panics when `T` is accessed by another thread.
    ///    This implementation is used when `#[pyclass(unsendable)]` is given.
    ///    Panicking makes it safe to expose `T: !Send` to the Python interpreter, where all objects
    ///    can be accessed by multiple threads by `threading` module.
    type ThreadChecker: PyClassThreadChecker<Self>;

    #[cfg(feature = "multiple-pymethods")]
    type Inventory: PyClassInventory;

    /// Rendered class doc
    fn doc(py: Python<'_>) -> PyResult<&'static CStr>;

    fn items_iter() -> PyClassItemsIter;

    #[inline]
    fn dict_offset() -> Option<ffi::Py_ssize_t> {
        None
    }

    #[inline]
    fn weaklist_offset() -> Option<ffi::Py_ssize_t> {
        None
    }

    fn lazy_type_object() -> &'static LazyTypeObject<Self>;
}

/// Runtime helper to build a class docstring from the `doc` and `text_signature`.
///
/// This is done at runtime because the class text signature is collected via dtolnay
/// specialization in to the `#[pyclass]` macro from the `#[pymethods]` macro.
pub fn build_pyclass_doc(
    class_name: &'static str,
    doc: &'static str,
    text_signature: Option<&'static str>,
) -> PyResult<Cow<'static, CStr>> {
    if let Some(text_signature) = text_signature {
        let doc = CString::new(format!(
            "{}{}\n--\n\n{}",
            class_name,
            text_signature,
            doc.trim_end_matches('\0')
        ))
        .map_err(|_| PyValueError::new_err("class doc cannot contain nul bytes"))?;
        Ok(Cow::Owned(doc))
    } else {
        extract_c_string(doc, "class doc cannot contain nul bytes")
    }
}

/// Iterator used to process all class items during type instantiation.
pub struct PyClassItemsIter {
    /// Iteration state
    idx: usize,
    /// Items from the `#[pyclass]` macro
    pyclass_items: &'static PyClassItems,
    /// Items from the `#[pymethods]` macro
    #[cfg(not(feature = "multiple-pymethods"))]
    pymethods_items: &'static PyClassItems,
    /// Items from the `#[pymethods]` macro with inventory
    #[cfg(feature = "multiple-pymethods")]
    pymethods_items: Box<dyn Iterator<Item = &'static PyClassItems>>,
}

impl PyClassItemsIter {
    pub fn new(
        pyclass_items: &'static PyClassItems,
        #[cfg(not(feature = "multiple-pymethods"))] pymethods_items: &'static PyClassItems,
        #[cfg(feature = "multiple-pymethods")] pymethods_items: Box<
            dyn Iterator<Item = &'static PyClassItems>,
        >,
    ) -> Self {
        Self {
            idx: 0,
            pyclass_items,
            pymethods_items,
        }
    }
}

impl Iterator for PyClassItemsIter {
    type Item = &'static PyClassItems;

    #[cfg(not(feature = "multiple-pymethods"))]
    fn next(&mut self) -> Option<Self::Item> {
        match self.idx {
            0 => {
                self.idx += 1;
                Some(self.pyclass_items)
            }
            1 => {
                self.idx += 1;
                Some(self.pymethods_items)
            }
            // Termination clause
            _ => None,
        }
    }

    #[cfg(feature = "multiple-pymethods")]
    fn next(&mut self) -> Option<Self::Item> {
        match self.idx {
            0 => {
                self.idx += 1;
                Some(self.pyclass_items)
            }
            // Termination clause
            _ => self.pymethods_items.next(),
        }
    }
}

// Traits describing known special methods.

macro_rules! slot_fragment_trait {
    ($trait_name:ident, $($default_method:tt)*) => {
        #[allow(non_camel_case_types)]
        pub trait $trait_name<T>: Sized {
            $($default_method)*
        }

        impl<T> $trait_name<T> for &'_ PyClassImplCollector<T> {}
    }
}

slot_fragment_trait! {
    PyClass__getattribute__SlotFragment,

    /// # Safety: _slf and _attr must be valid non-null Python objects
    #[inline]
    unsafe fn __getattribute__(
        self,
        py: Python<'_>,
        slf: *mut ffi::PyObject,
        attr: *mut ffi::PyObject,
    ) -> PyResult<*mut ffi::PyObject> {
        let res = ffi::PyObject_GenericGetAttr(slf, attr);
        if res.is_null() {
            Err(PyErr::fetch(py))
        } else {
            Ok(res)
        }
    }
}

slot_fragment_trait! {
    PyClass__getattr__SlotFragment,

    /// # Safety: _slf and _attr must be valid non-null Python objects
    #[inline]
    unsafe fn __getattr__(
        self,
        py: Python<'_>,
        _slf: *mut ffi::PyObject,
        attr: *mut ffi::PyObject,
    ) -> PyResult<*mut ffi::PyObject> {
        Err(PyErr::new::<PyAttributeError, _>(
            (Py::<PyAny>::from_borrowed_ptr(py, attr),)
        ))
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! generate_pyclass_getattro_slot {
    ($cls:ty) => {{
        unsafe extern "C" fn __wrap(
            _slf: *mut $crate::ffi::PyObject,
            attr: *mut $crate::ffi::PyObject,
        ) -> *mut $crate::ffi::PyObject {
            $crate::impl_::trampoline::getattrofunc(_slf, attr, |py, _slf, attr| {
                use ::std::result::Result::*;
                use $crate::impl_::pyclass::*;
                let collector = PyClassImplCollector::<$cls>::new();

                // Strategy:
                // - Try __getattribute__ first. Its default is PyObject_GenericGetAttr.
                // - If it returns a result, use it.
                // - If it fails with AttributeError, try __getattr__.
                // - If it fails otherwise, reraise.
                match collector.__getattribute__(py, _slf, attr) {
                    Ok(obj) => Ok(obj),
                    Err(e) if e.is_instance_of::<$crate::exceptions::PyAttributeError>(py) => {
                        collector.__getattr__(py, _slf, attr)
                    }
                    Err(e) => Err(e),
                }
            })
        }
        $crate::ffi::PyType_Slot {
            slot: $crate::ffi::Py_tp_getattro,
            pfunc: __wrap as $crate::ffi::getattrofunc as _,
        }
    }};
}

pub use generate_pyclass_getattro_slot;

/// Macro which expands to three items
/// - Trait for a __setitem__ dunder
/// - Trait for the corresponding __delitem__ dunder
/// - A macro which will use dtolnay specialisation to generate the shared slot for the two dunders
macro_rules! define_pyclass_setattr_slot {
    (
        $set_trait:ident,
        $del_trait:ident,
        $set:ident,
        $del:ident,
        $set_error:expr,
        $del_error:expr,
        $generate_macro:ident,
        $slot:ident,
        $func_ty:ident,
    ) => {
        slot_fragment_trait! {
            $set_trait,

            /// # Safety: _slf and _attr must be valid non-null Python objects
            #[inline]
            unsafe fn $set(
                self,
                _py: Python<'_>,
                _slf: *mut ffi::PyObject,
                _attr: *mut ffi::PyObject,
                _value: NonNull<ffi::PyObject>,
            ) -> PyResult<()> {
                $set_error
            }
        }

        slot_fragment_trait! {
            $del_trait,

            /// # Safety: _slf and _attr must be valid non-null Python objects
            #[inline]
            unsafe fn $del(
                self,
                _py: Python<'_>,
                _slf: *mut ffi::PyObject,
                _attr: *mut ffi::PyObject,
            ) -> PyResult<()> {
                $del_error
            }
        }

        #[doc(hidden)]
        #[macro_export]
        macro_rules! $generate_macro {
            ($cls:ty) => {{
                unsafe extern "C" fn __wrap(
                    _slf: *mut $crate::ffi::PyObject,
                    attr: *mut $crate::ffi::PyObject,
                    value: *mut $crate::ffi::PyObject,
                ) -> ::std::os::raw::c_int {
                    $crate::impl_::trampoline::setattrofunc(
                        _slf,
                        attr,
                        value,
                        |py, _slf, attr, value| {
                            use ::std::option::Option::*;
                            use $crate::callback::IntoPyCallbackOutput;
                            use $crate::impl_::pyclass::*;
                            let collector = PyClassImplCollector::<$cls>::new();
                            if let Some(value) = ::std::ptr::NonNull::new(value) {
                                collector.$set(py, _slf, attr, value).convert(py)
                            } else {
                                collector.$del(py, _slf, attr).convert(py)
                            }
                        },
                    )
                }
                $crate::ffi::PyType_Slot {
                    slot: $crate::ffi::$slot,
                    pfunc: __wrap as $crate::ffi::$func_ty as _,
                }
            }};
        }
        pub use $generate_macro;
    };
}

define_pyclass_setattr_slot! {
    PyClass__setattr__SlotFragment,
    PyClass__delattr__SlotFragment,
    __setattr__,
    __delattr__,
    Err(PyAttributeError::new_err("can't set attribute")),
    Err(PyAttributeError::new_err("can't delete attribute")),
    generate_pyclass_setattr_slot,
    Py_tp_setattro,
    setattrofunc,
}

define_pyclass_setattr_slot! {
    PyClass__set__SlotFragment,
    PyClass__delete__SlotFragment,
    __set__,
    __delete__,
    Err(PyNotImplementedError::new_err("can't set descriptor")),
    Err(PyNotImplementedError::new_err("can't delete descriptor")),
    generate_pyclass_setdescr_slot,
    Py_tp_descr_set,
    descrsetfunc,
}

define_pyclass_setattr_slot! {
    PyClass__setitem__SlotFragment,
    PyClass__delitem__SlotFragment,
    __setitem__,
    __delitem__,
    Err(PyNotImplementedError::new_err("can't set item")),
    Err(PyNotImplementedError::new_err("can't delete item")),
    generate_pyclass_setitem_slot,
    Py_mp_ass_subscript,
    objobjargproc,
}

/// Macro which expands to three items
/// - Trait for a lhs dunder e.g. __add__
/// - Trait for the corresponding rhs e.g. __radd__
/// - A macro which will use dtolnay specialisation to generate the shared slot for the two dunders
macro_rules! define_pyclass_binary_operator_slot {
    (
        $lhs_trait:ident,
        $rhs_trait:ident,
        $lhs:ident,
        $rhs:ident,
        $generate_macro:ident,
        $slot:ident,
        $func_ty:ident,
    ) => {
        slot_fragment_trait! {
            $lhs_trait,

            /// # Safety: _slf and _other must be valid non-null Python objects
            #[inline]
            unsafe fn $lhs(
                self,
                _py: Python<'_>,
                _slf: *mut ffi::PyObject,
                _other: *mut ffi::PyObject,
            ) -> PyResult<*mut ffi::PyObject> {
                Ok(ffi::_Py_NewRef(ffi::Py_NotImplemented()))
            }
        }

        slot_fragment_trait! {
            $rhs_trait,

            /// # Safety: _slf and _other must be valid non-null Python objects
            #[inline]
            unsafe fn $rhs(
                self,
                _py: Python<'_>,
                _slf: *mut ffi::PyObject,
                _other: *mut ffi::PyObject,
            ) -> PyResult<*mut ffi::PyObject> {
                Ok(ffi::_Py_NewRef(ffi::Py_NotImplemented()))
            }
        }

        #[doc(hidden)]
        #[macro_export]
        macro_rules! $generate_macro {
            ($cls:ty) => {{
                unsafe extern "C" fn __wrap(
                    _slf: *mut $crate::ffi::PyObject,
                    _other: *mut $crate::ffi::PyObject,
                ) -> *mut $crate::ffi::PyObject {
                    $crate::impl_::trampoline::binaryfunc(_slf, _other, |py, _slf, _other| {
                        use $crate::impl_::pyclass::*;
                        let collector = PyClassImplCollector::<$cls>::new();
                        let lhs_result = collector.$lhs(py, _slf, _other)?;
                        if lhs_result == $crate::ffi::Py_NotImplemented() {
                            $crate::ffi::Py_DECREF(lhs_result);
                            collector.$rhs(py, _other, _slf)
                        } else {
                            ::std::result::Result::Ok(lhs_result)
                        }
                    })
                }
                $crate::ffi::PyType_Slot {
                    slot: $crate::ffi::$slot,
                    pfunc: __wrap as $crate::ffi::$func_ty as _,
                }
            }};
        }
        pub use $generate_macro;
    };
}

define_pyclass_binary_operator_slot! {
    PyClass__add__SlotFragment,
    PyClass__radd__SlotFragment,
    __add__,
    __radd__,
    generate_pyclass_add_slot,
    Py_nb_add,
    binaryfunc,
}

define_pyclass_binary_operator_slot! {
    PyClass__sub__SlotFragment,
    PyClass__rsub__SlotFragment,
    __sub__,
    __rsub__,
    generate_pyclass_sub_slot,
    Py_nb_subtract,
    binaryfunc,
}

define_pyclass_binary_operator_slot! {
    PyClass__mul__SlotFragment,
    PyClass__rmul__SlotFragment,
    __mul__,
    __rmul__,
    generate_pyclass_mul_slot,
    Py_nb_multiply,
    binaryfunc,
}

define_pyclass_binary_operator_slot! {
    PyClass__mod__SlotFragment,
    PyClass__rmod__SlotFragment,
    __mod__,
    __rmod__,
    generate_pyclass_mod_slot,
    Py_nb_remainder,
    binaryfunc,
}

define_pyclass_binary_operator_slot! {
    PyClass__divmod__SlotFragment,
    PyClass__rdivmod__SlotFragment,
    __divmod__,
    __rdivmod__,
    generate_pyclass_divmod_slot,
    Py_nb_divmod,
    binaryfunc,
}

define_pyclass_binary_operator_slot! {
    PyClass__lshift__SlotFragment,
    PyClass__rlshift__SlotFragment,
    __lshift__,
    __rlshift__,
    generate_pyclass_lshift_slot,
    Py_nb_lshift,
    binaryfunc,
}

define_pyclass_binary_operator_slot! {
    PyClass__rshift__SlotFragment,
    PyClass__rrshift__SlotFragment,
    __rshift__,
    __rrshift__,
    generate_pyclass_rshift_slot,
    Py_nb_rshift,
    binaryfunc,
}

define_pyclass_binary_operator_slot! {
    PyClass__and__SlotFragment,
    PyClass__rand__SlotFragment,
    __and__,
    __rand__,
    generate_pyclass_and_slot,
    Py_nb_and,
    binaryfunc,
}

define_pyclass_binary_operator_slot! {
    PyClass__or__SlotFragment,
    PyClass__ror__SlotFragment,
    __or__,
    __ror__,
    generate_pyclass_or_slot,
    Py_nb_or,
    binaryfunc,
}

define_pyclass_binary_operator_slot! {
    PyClass__xor__SlotFragment,
    PyClass__rxor__SlotFragment,
    __xor__,
    __rxor__,
    generate_pyclass_xor_slot,
    Py_nb_xor,
    binaryfunc,
}

define_pyclass_binary_operator_slot! {
    PyClass__matmul__SlotFragment,
    PyClass__rmatmul__SlotFragment,
    __matmul__,
    __rmatmul__,
    generate_pyclass_matmul_slot,
    Py_nb_matrix_multiply,
    binaryfunc,
}

define_pyclass_binary_operator_slot! {
    PyClass__truediv__SlotFragment,
    PyClass__rtruediv__SlotFragment,
    __truediv__,
    __rtruediv__,
    generate_pyclass_truediv_slot,
    Py_nb_true_divide,
    binaryfunc,
}

define_pyclass_binary_operator_slot! {
    PyClass__floordiv__SlotFragment,
    PyClass__rfloordiv__SlotFragment,
    __floordiv__,
    __rfloordiv__,
    generate_pyclass_floordiv_slot,
    Py_nb_floor_divide,
    binaryfunc,
}

slot_fragment_trait! {
    PyClass__pow__SlotFragment,

    /// # Safety: _slf and _other must be valid non-null Python objects
    #[inline]
    unsafe fn __pow__(
        self,
        _py: Python<'_>,
        _slf: *mut ffi::PyObject,
        _other: *mut ffi::PyObject,
        _mod: *mut ffi::PyObject,
    ) -> PyResult<*mut ffi::PyObject> {
        Ok(ffi::_Py_NewRef(ffi::Py_NotImplemented()))
    }
}

slot_fragment_trait! {
    PyClass__rpow__SlotFragment,

    /// # Safety: _slf and _other must be valid non-null Python objects
    #[inline]
    unsafe fn __rpow__(
        self,
        _py: Python<'_>,
        _slf: *mut ffi::PyObject,
        _other: *mut ffi::PyObject,
        _mod: *mut ffi::PyObject,
    ) -> PyResult<*mut ffi::PyObject> {
        Ok(ffi::_Py_NewRef(ffi::Py_NotImplemented()))
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! generate_pyclass_pow_slot {
    ($cls:ty) => {{
        unsafe extern "C" fn __wrap(
            _slf: *mut $crate::ffi::PyObject,
            _other: *mut $crate::ffi::PyObject,
            _mod: *mut $crate::ffi::PyObject,
        ) -> *mut $crate::ffi::PyObject {
            $crate::impl_::trampoline::ternaryfunc(_slf, _other, _mod, |py, _slf, _other, _mod| {
                use $crate::impl_::pyclass::*;
                let collector = PyClassImplCollector::<$cls>::new();
                let lhs_result = collector.__pow__(py, _slf, _other, _mod)?;
                if lhs_result == $crate::ffi::Py_NotImplemented() {
                    $crate::ffi::Py_DECREF(lhs_result);
                    collector.__rpow__(py, _other, _slf, _mod)
                } else {
                    ::std::result::Result::Ok(lhs_result)
                }
            })
        }
        $crate::ffi::PyType_Slot {
            slot: $crate::ffi::Py_nb_power,
            pfunc: __wrap as $crate::ffi::ternaryfunc as _,
        }
    }};
}
pub use generate_pyclass_pow_slot;

slot_fragment_trait! {
    PyClass__lt__SlotFragment,

    /// # Safety: _slf and _other must be valid non-null Python objects
    #[inline]
    unsafe fn __lt__(
        self,
        _py: Python<'_>,
        _slf: *mut ffi::PyObject,
        _other: *mut ffi::PyObject,
    ) -> PyResult<*mut ffi::PyObject> {
        Ok(ffi::_Py_NewRef(ffi::Py_NotImplemented()))
    }
}

slot_fragment_trait! {
    PyClass__le__SlotFragment,

    /// # Safety: _slf and _other must be valid non-null Python objects
    #[inline]
    unsafe fn __le__(
        self,
        _py: Python<'_>,
        _slf: *mut ffi::PyObject,
        _other: *mut ffi::PyObject,
    ) -> PyResult<*mut ffi::PyObject> {
        Ok(ffi::_Py_NewRef(ffi::Py_NotImplemented()))
    }
}

slot_fragment_trait! {
    PyClass__eq__SlotFragment,

    /// # Safety: _slf and _other must be valid non-null Python objects
    #[inline]
    unsafe fn __eq__(
        self,
        _py: Python<'_>,
        _slf: *mut ffi::PyObject,
        _other: *mut ffi::PyObject,
    ) -> PyResult<*mut ffi::PyObject> {
        Ok(ffi::_Py_NewRef(ffi::Py_NotImplemented()))
    }
}

slot_fragment_trait! {
    PyClass__ne__SlotFragment,

    /// # Safety: _slf and _other must be valid non-null Python objects
    #[inline]
    unsafe fn __ne__(
        self,
        py: Python<'_>,
        slf: *mut ffi::PyObject,
        other: *mut ffi::PyObject,
    ) -> PyResult<*mut ffi::PyObject> {
        // By default `__ne__` will try `__eq__` and invert the result
        let slf: &PyAny = py.from_borrowed_ptr(slf);
        let other: &PyAny = py.from_borrowed_ptr(other);
        slf.eq(other).map(|is_eq| PyBool::new(py, !is_eq).into_ptr())
    }
}

slot_fragment_trait! {
    PyClass__gt__SlotFragment,

    /// # Safety: _slf and _other must be valid non-null Python objects
    #[inline]
    unsafe fn __gt__(
        self,
        _py: Python<'_>,
        _slf: *mut ffi::PyObject,
        _other: *mut ffi::PyObject,
    ) -> PyResult<*mut ffi::PyObject> {
        Ok(ffi::_Py_NewRef(ffi::Py_NotImplemented()))
    }
}

slot_fragment_trait! {
    PyClass__ge__SlotFragment,

    /// # Safety: _slf and _other must be valid non-null Python objects
    #[inline]
    unsafe fn __ge__(
        self,
        _py: Python<'_>,
        _slf: *mut ffi::PyObject,
        _other: *mut ffi::PyObject,
    ) -> PyResult<*mut ffi::PyObject> {
        Ok(ffi::_Py_NewRef(ffi::Py_NotImplemented()))
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! generate_pyclass_richcompare_slot {
    ($cls:ty) => {{
        impl $cls {
            #[allow(non_snake_case)]
            unsafe extern "C" fn __pymethod___richcmp____(
                slf: *mut $crate::ffi::PyObject,
                other: *mut $crate::ffi::PyObject,
                op: ::std::os::raw::c_int,
            ) -> *mut $crate::ffi::PyObject {
                $crate::impl_::trampoline::richcmpfunc(slf, other, op, |py, slf, other, op| {
                    use $crate::class::basic::CompareOp;
                    use $crate::impl_::pyclass::*;
                    let collector = PyClassImplCollector::<$cls>::new();
                    match CompareOp::from_raw(op).expect("invalid compareop") {
                        CompareOp::Lt => collector.__lt__(py, slf, other),
                        CompareOp::Le => collector.__le__(py, slf, other),
                        CompareOp::Eq => collector.__eq__(py, slf, other),
                        CompareOp::Ne => collector.__ne__(py, slf, other),
                        CompareOp::Gt => collector.__gt__(py, slf, other),
                        CompareOp::Ge => collector.__ge__(py, slf, other),
                    }
                })
            }
        }
        $crate::ffi::PyType_Slot {
            slot: $crate::ffi::Py_tp_richcompare,
            pfunc: <$cls>::__pymethod___richcmp____ as $crate::ffi::richcmpfunc as _,
        }
    }};
}
pub use generate_pyclass_richcompare_slot;

/// Implements a freelist.
///
/// Do not implement this trait manually. Instead, use `#[pyclass(freelist = N)]`
/// on a Rust struct to implement it.
pub trait PyClassWithFreeList: PyClass {
    fn get_free_list(py: Python<'_>) -> &mut FreeList<*mut ffi::PyObject>;
}

/// Implementation of tp_alloc for `freelist` classes.
///
/// # Safety
/// - `subtype` must be a valid pointer to the type object of T or a subclass.
/// - The GIL must be held.
pub unsafe extern "C" fn alloc_with_freelist<T: PyClassWithFreeList>(
    subtype: *mut ffi::PyTypeObject,
    nitems: ffi::Py_ssize_t,
) -> *mut ffi::PyObject {
    let py = Python::assume_gil_acquired();

    #[cfg(not(Py_3_8))]
    bpo_35810_workaround(py, subtype);

    let self_type = T::type_object_raw(py);
    // If this type is a variable type or the subtype is not equal to this type, we cannot use the
    // freelist
    if nitems == 0 && subtype == self_type {
        if let Some(obj) = T::get_free_list(py).pop() {
            ffi::PyObject_Init(obj, subtype);
            return obj as _;
        }
    }

    ffi::PyType_GenericAlloc(subtype, nitems)
}

/// Implementation of tp_free for `freelist` classes.
///
/// # Safety
/// - `obj` must be a valid pointer to an instance of T (not a subclass).
/// - The GIL must be held.
pub unsafe extern "C" fn free_with_freelist<T: PyClassWithFreeList>(obj: *mut c_void) {
    let obj = obj as *mut ffi::PyObject;
    debug_assert_eq!(
        T::type_object_raw(Python::assume_gil_acquired()),
        ffi::Py_TYPE(obj)
    );
    if let Some(obj) = T::get_free_list(Python::assume_gil_acquired()).insert(obj) {
        let ty = ffi::Py_TYPE(obj);

        // Deduce appropriate inverse of PyType_GenericAlloc
        let free = if ffi::PyType_IS_GC(ty) != 0 {
            ffi::PyObject_GC_Del
        } else {
            ffi::PyObject_Free
        };
        free(obj as *mut c_void);

        #[cfg(Py_3_8)]
        if ffi::PyType_HasFeature(ty, ffi::Py_TPFLAGS_HEAPTYPE) != 0 {
            ffi::Py_DECREF(ty as *mut ffi::PyObject);
        }
    }
}

/// Workaround for Python issue 35810; no longer necessary in Python 3.8
#[inline]
#[cfg(not(Py_3_8))]
unsafe fn bpo_35810_workaround(py: Python<'_>, ty: *mut ffi::PyTypeObject) {
    #[cfg(Py_LIMITED_API)]
    {
        // Must check version at runtime for abi3 wheels - they could run against a higher version
        // than the build config suggests.
        use crate::sync::GILOnceCell;
        static IS_PYTHON_3_8: GILOnceCell<bool> = GILOnceCell::new();

        if *IS_PYTHON_3_8.get_or_init(py, || py.version_info() >= (3, 8)) {
            // No fix needed - the wheel is running on a sufficiently new interpreter.
            return;
        }
    }
    #[cfg(not(Py_LIMITED_API))]
    {
        // suppress unused variable warning
        let _ = py;
    }

    ffi::Py_INCREF(ty as *mut ffi::PyObject);
}

/// Implementation detail. Only to be used through our proc macro code.
/// Method storage for `#[pyclass]`.
/// Allows arbitrary `#[pymethod]` blocks to submit their methods,
/// which are eventually collected by `#[pyclass]`.
#[cfg(feature = "multiple-pymethods")]
pub trait PyClassInventory: inventory::Collect {
    /// Returns the items for a single `#[pymethods] impl` block
    fn items(&'static self) -> &'static PyClassItems;
}

// Items from #[pymethods] if not using inventory.
#[cfg(not(feature = "multiple-pymethods"))]
pub trait PyMethods<T> {
    fn py_methods(self) -> &'static PyClassItems;
}

#[cfg(not(feature = "multiple-pymethods"))]
impl<T> PyMethods<T> for &'_ PyClassImplCollector<T> {
    fn py_methods(self) -> &'static PyClassItems {
        &PyClassItems {
            methods: &[],
            slots: &[],
        }
    }
}

// Text signature for __new__
pub trait PyClassNewTextSignature<T> {
    fn new_text_signature(self) -> Option<&'static str>;
}

impl<T> PyClassNewTextSignature<T> for &'_ PyClassImplCollector<T> {
    #[inline]
    fn new_text_signature(self) -> Option<&'static str> {
        None
    }
}

// Thread checkers

#[doc(hidden)]
pub trait PyClassThreadChecker<T>: Sized {
    fn ensure(&self);
    fn can_drop(&self, py: Python<'_>) -> bool;
    fn new() -> Self;
    private_decl! {}
}

/// Default thread checker for `#[pyclass]`.
///
/// Keeping the T: Send bound here slightly improves the compile
/// error message to hint to users to figure out what's wrong
/// when `#[pyclass]` types do not implement `Send`.
#[doc(hidden)]
pub struct SendablePyClass<T: Send>(PhantomData<T>);

impl<T: Send> PyClassThreadChecker<T> for SendablePyClass<T> {
    fn ensure(&self) {}
    fn can_drop(&self, _py: Python<'_>) -> bool {
        true
    }
    #[inline]
    fn new() -> Self {
        SendablePyClass(PhantomData)
    }
    private_impl! {}
}

/// Thread checker for `#[pyclass(unsendable)]` types.
/// Panics when the value is accessed by another thread.
#[doc(hidden)]
pub struct ThreadCheckerImpl(thread::ThreadId);

impl ThreadCheckerImpl {
    fn ensure(&self, type_name: &'static str) {
        assert_eq!(
            thread::current().id(),
            self.0,
            "{} is unsendable, but sent to another thread",
            type_name
        );
    }

    fn can_drop(&self, py: Python<'_>, type_name: &'static str) -> bool {
        if thread::current().id() != self.0 {
            PyRuntimeError::new_err(format!(
                "{} is unsendable, but is being dropped on another thread",
                type_name
            ))
            .write_unraisable(py, None);
            return false;
        }

        true
    }
}

impl<T> PyClassThreadChecker<T> for ThreadCheckerImpl {
    fn ensure(&self) {
        self.ensure(std::any::type_name::<T>());
    }
    fn can_drop(&self, py: Python<'_>) -> bool {
        self.can_drop(py, std::any::type_name::<T>())
    }
    fn new() -> Self {
        ThreadCheckerImpl(thread::current().id())
    }
    private_impl! {}
}

/// Trait denoting that this class is suitable to be used as a base type for PyClass.
pub trait PyClassBaseType: Sized {
    type LayoutAsBase: PyCellLayout<Self>;
    type BaseNativeType;
    type Initializer: PyObjectInit<Self>;
    type PyClassMutability: PyClassMutability;
}

/// All mutable PyClasses can be used as a base type.
///
/// In the future this will be extended to immutable PyClasses too.
impl<T: PyClass> PyClassBaseType for T {
    type LayoutAsBase = crate::pycell::PyCell<T>;
    type BaseNativeType = T::BaseNativeType;
    type Initializer = crate::pyclass_init::PyClassInitializer<Self>;
    type PyClassMutability = T::PyClassMutability;
}

/// Implementation of tp_dealloc for pyclasses without gc
pub(crate) unsafe extern "C" fn tp_dealloc<T: PyClass>(obj: *mut ffi::PyObject) {
    crate::impl_::trampoline::dealloc(obj, PyCell::<T>::tp_dealloc)
}

/// Implementation of tp_dealloc for pyclasses with gc
pub(crate) unsafe extern "C" fn tp_dealloc_with_gc<T: PyClass>(obj: *mut ffi::PyObject) {
    #[cfg(not(PyPy))]
    {
        ffi::PyObject_GC_UnTrack(obj.cast());
    }
    crate::impl_::trampoline::dealloc(obj, PyCell::<T>::tp_dealloc)
}

pub(crate) unsafe extern "C" fn get_sequence_item_from_mapping(
    obj: *mut ffi::PyObject,
    index: ffi::Py_ssize_t,
) -> *mut ffi::PyObject {
    let index = ffi::PyLong_FromSsize_t(index);
    if index.is_null() {
        return std::ptr::null_mut();
    }
    let result = ffi::PyObject_GetItem(obj, index);
    ffi::Py_DECREF(index);
    result
}

pub(crate) unsafe extern "C" fn assign_sequence_item_from_mapping(
    obj: *mut ffi::PyObject,
    index: ffi::Py_ssize_t,
    value: *mut ffi::PyObject,
) -> c_int {
    let index = ffi::PyLong_FromSsize_t(index);
    if index.is_null() {
        return -1;
    }
    let result = if value.is_null() {
        ffi::PyObject_DelItem(obj, index)
    } else {
        ffi::PyObject_SetItem(obj, index, value)
    };
    ffi::Py_DECREF(index);
    result
}
