use std::iter::FusedIterator;

use crate::err::{self, PyResult};
use crate::ffi::{self, Py_ssize_t};
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::instance::Borrowed;
use crate::internal_tricks::get_ssize_index;
use crate::types::{PySequence, PyTuple};
use crate::{Bound, PyAny, PyNativeType, PyObject, Python, ToPyObject};

use crate::types::any::PyAnyMethods;
use crate::types::sequence::PySequenceMethods;

/// Represents a Python `list`.
#[repr(transparent)]
pub struct PyList(PyAny);

pyobject_native_type_core!(PyList, pyobject_native_static_type_object!(ffi::PyList_Type), #checkfunction=ffi::PyList_Check);

#[inline]
#[track_caller]
pub(crate) fn new_from_iter<'py>(
    py: Python<'py>,
    elements: &mut dyn ExactSizeIterator<Item = PyObject>,
) -> Bound<'py, PyList> {
    unsafe {
        // PyList_New checks for overflow but has a bad error message, so we check ourselves
        let len: Py_ssize_t = elements
            .len()
            .try_into()
            .expect("out of range integral type conversion attempted on `elements.len()`");

        let ptr = ffi::PyList_New(len);

        // We create the `Bound` pointer here for two reasons:
        // - panics if the ptr is null
        // - its Drop cleans up the list if user code or the asserts panic.
        let list = ptr.assume_owned(py).downcast_into_unchecked();

        let mut counter: Py_ssize_t = 0;

        for obj in elements.take(len as usize) {
            #[cfg(not(Py_LIMITED_API))]
            ffi::PyList_SET_ITEM(ptr, counter, obj.into_ptr());
            #[cfg(Py_LIMITED_API)]
            ffi::PyList_SetItem(ptr, counter, obj.into_ptr());
            counter += 1;
        }

        assert!(elements.next().is_none(), "Attempted to create PyList but `elements` was larger than reported by its `ExactSizeIterator` implementation.");
        assert_eq!(len, counter, "Attempted to create PyList but `elements` was smaller than reported by its `ExactSizeIterator` implementation.");

        list
    }
}

impl PyList {
    /// Deprecated form of [`PyList::new_bound`].
    #[inline]
    #[track_caller]
    #[cfg_attr(
        not(feature = "gil-refs"),
        deprecated(
            since = "0.21.0",
            note = "`PyList::new` will be replaced by `PyList::new_bound` in a future PyO3 version"
        )
    )]
    pub fn new<T, U>(py: Python<'_>, elements: impl IntoIterator<Item = T, IntoIter = U>) -> &PyList
    where
        T: ToPyObject,
        U: ExactSizeIterator<Item = T>,
    {
        Self::new_bound(py, elements).into_gil_ref()
    }

    /// Constructs a new list with the given elements.
    ///
    /// If you want to create a [`PyList`] with elements of different or unknown types, or from an
    /// iterable that doesn't implement [`ExactSizeIterator`], use [`PyList::append`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pyo3::prelude::*;
    /// use pyo3::types::PyList;
    ///
    /// # fn main() {
    /// Python::with_gil(|py| {
    ///     let elements: Vec<i32> = vec![0, 1, 2, 3, 4, 5];
    ///     let list = PyList::new_bound(py, elements);
    ///     assert_eq!(format!("{:?}", list), "[0, 1, 2, 3, 4, 5]");
    /// });
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// This function will panic if `element`'s [`ExactSizeIterator`] implementation is incorrect.
    /// All standard library structures implement this trait correctly, if they do, so calling this
    /// function with (for example) [`Vec`]`<T>` or `&[T]` will always succeed.
    #[track_caller]
    pub fn new_bound<T, U>(
        py: Python<'_>,
        elements: impl IntoIterator<Item = T, IntoIter = U>,
    ) -> Bound<'_, PyList>
    where
        T: ToPyObject,
        U: ExactSizeIterator<Item = T>,
    {
        let mut iter = elements.into_iter().map(|e| e.to_object(py));
        new_from_iter(py, &mut iter)
    }

    /// Deprecated form of [`PyList::empty_bound`].
    #[inline]
    #[cfg_attr(
        not(feature = "gil-refs"),
        deprecated(
            since = "0.21.0",
            note = "`PyList::empty` will be replaced by `PyList::empty_bound` in a future PyO3 version"
        )
    )]
    pub fn empty(py: Python<'_>) -> &PyList {
        Self::empty_bound(py).into_gil_ref()
    }

    /// Constructs a new empty list.
    pub fn empty_bound(py: Python<'_>) -> Bound<'_, PyList> {
        unsafe {
            ffi::PyList_New(0)
                .assume_owned(py)
                .downcast_into_unchecked()
        }
    }

    /// Returns the length of the list.
    pub fn len(&self) -> usize {
        self.as_borrowed().len()
    }

    /// Checks if the list is empty.
    pub fn is_empty(&self) -> bool {
        self.as_borrowed().is_empty()
    }

    /// Returns `self` cast as a `PySequence`.
    pub fn as_sequence(&self) -> &PySequence {
        unsafe { self.downcast_unchecked() }
    }

    /// Gets the list item at the specified index.
    /// # Example
    /// ```
    /// use pyo3::{prelude::*, types::PyList};
    /// Python::with_gil(|py| {
    ///     let list = PyList::new_bound(py, [2, 3, 5, 7]);
    ///     let obj = list.get_item(0);
    ///     assert_eq!(obj.unwrap().extract::<i32>().unwrap(), 2);
    /// });
    /// ```
    pub fn get_item(&self, index: usize) -> PyResult<&PyAny> {
        self.as_borrowed().get_item(index).map(Bound::into_gil_ref)
    }

    /// Gets the list item at the specified index. Undefined behavior on bad index. Use with caution.
    ///
    /// # Safety
    ///
    /// Caller must verify that the index is within the bounds of the list.
    #[cfg(not(Py_LIMITED_API))]
    pub unsafe fn get_item_unchecked(&self, index: usize) -> &PyAny {
        self.as_borrowed().get_item_unchecked(index).into_gil_ref()
    }

    /// Takes the slice `self[low:high]` and returns it as a new list.
    ///
    /// Indices must be nonnegative, and out-of-range indices are clipped to
    /// `self.len()`.
    pub fn get_slice(&self, low: usize, high: usize) -> &PyList {
        self.as_borrowed().get_slice(low, high).into_gil_ref()
    }

    /// Sets the item at the specified index.
    ///
    /// Raises `IndexError` if the index is out of range.
    pub fn set_item<I>(&self, index: usize, item: I) -> PyResult<()>
    where
        I: ToPyObject,
    {
        self.as_borrowed().set_item(index, item)
    }

    /// Deletes the `index`th element of self.
    ///
    /// This is equivalent to the Python statement `del self[i]`.
    #[inline]
    pub fn del_item(&self, index: usize) -> PyResult<()> {
        self.as_borrowed().del_item(index)
    }

    /// Assigns the sequence `seq` to the slice of `self` from `low` to `high`.
    ///
    /// This is equivalent to the Python statement `self[low:high] = v`.
    #[inline]
    pub fn set_slice(&self, low: usize, high: usize, seq: &PyAny) -> PyResult<()> {
        self.as_borrowed().set_slice(low, high, &seq.as_borrowed())
    }

    /// Deletes the slice from `low` to `high` from `self`.
    ///
    /// This is equivalent to the Python statement `del self[low:high]`.
    #[inline]
    pub fn del_slice(&self, low: usize, high: usize) -> PyResult<()> {
        self.as_borrowed().del_slice(low, high)
    }

    /// Appends an item to the list.
    pub fn append<I>(&self, item: I) -> PyResult<()>
    where
        I: ToPyObject,
    {
        self.as_borrowed().append(item)
    }

    /// Inserts an item at the specified index.
    ///
    /// If `index >= self.len()`, inserts at the end.
    pub fn insert<I>(&self, index: usize, item: I) -> PyResult<()>
    where
        I: ToPyObject,
    {
        self.as_borrowed().insert(index, item)
    }

    /// Determines if self contains `value`.
    ///
    /// This is equivalent to the Python expression `value in self`.
    #[inline]
    pub fn contains<V>(&self, value: V) -> PyResult<bool>
    where
        V: ToPyObject,
    {
        self.as_borrowed().contains(value)
    }

    /// Returns the first index `i` for which `self[i] == value`.
    ///
    /// This is equivalent to the Python expression `self.index(value)`.
    #[inline]
    pub fn index<V>(&self, value: V) -> PyResult<usize>
    where
        V: ToPyObject,
    {
        self.as_borrowed().index(value)
    }

    /// Returns an iterator over this list's items.
    pub fn iter(&self) -> PyListIterator<'_> {
        PyListIterator(self.as_borrowed().iter())
    }

    /// Sorts the list in-place. Equivalent to the Python expression `l.sort()`.
    pub fn sort(&self) -> PyResult<()> {
        self.as_borrowed().sort()
    }

    /// Reverses the list in-place. Equivalent to the Python expression `l.reverse()`.
    pub fn reverse(&self) -> PyResult<()> {
        self.as_borrowed().reverse()
    }

    /// Return a new tuple containing the contents of the list; equivalent to the Python expression `tuple(list)`.
    ///
    /// This method is equivalent to `self.as_sequence().to_tuple()` and faster than `PyTuple::new(py, this_list)`.
    pub fn to_tuple(&self) -> &PyTuple {
        self.as_borrowed().to_tuple().into_gil_ref()
    }
}

index_impls!(PyList, "list", PyList::len, PyList::get_slice);

/// Implementation of functionality for [`PyList`].
///
/// These methods are defined for the `Bound<'py, PyList>` smart pointer, so to use method call
/// syntax these methods are separated into a trait, because stable Rust does not yet support
/// `arbitrary_self_types`.
#[doc(alias = "PyList")]
pub trait PyListMethods<'py>: crate::sealed::Sealed {
    /// Returns the length of the list.
    fn len(&self) -> usize;

    /// Checks if the list is empty.
    fn is_empty(&self) -> bool;

    /// Returns `self` cast as a `PySequence`.
    fn as_sequence(&self) -> &Bound<'py, PySequence>;

    /// Returns `self` cast as a `PySequence`.
    fn into_sequence(self) -> Bound<'py, PySequence>;

    /// Gets the list item at the specified index.
    /// # Example
    /// ```
    /// use pyo3::{prelude::*, types::PyList};
    /// Python::with_gil(|py| {
    ///     let list = PyList::new_bound(py, [2, 3, 5, 7]);
    ///     let obj = list.get_item(0);
    ///     assert_eq!(obj.unwrap().extract::<i32>().unwrap(), 2);
    /// });
    /// ```
    fn get_item(&self, index: usize) -> PyResult<Bound<'py, PyAny>>;

    /// Gets the list item at the specified index. Undefined behavior on bad index. Use with caution.
    ///
    /// # Safety
    ///
    /// Caller must verify that the index is within the bounds of the list.
    #[cfg(not(Py_LIMITED_API))]
    unsafe fn get_item_unchecked(&self, index: usize) -> Bound<'py, PyAny>;

    /// Takes the slice `self[low:high]` and returns it as a new list.
    ///
    /// Indices must be nonnegative, and out-of-range indices are clipped to
    /// `self.len()`.
    fn get_slice(&self, low: usize, high: usize) -> Bound<'py, PyList>;

    /// Sets the item at the specified index.
    ///
    /// Raises `IndexError` if the index is out of range.
    fn set_item<I>(&self, index: usize, item: I) -> PyResult<()>
    where
        I: ToPyObject;

    /// Deletes the `index`th element of self.
    ///
    /// This is equivalent to the Python statement `del self[i]`.
    fn del_item(&self, index: usize) -> PyResult<()>;

    /// Assigns the sequence `seq` to the slice of `self` from `low` to `high`.
    ///
    /// This is equivalent to the Python statement `self[low:high] = v`.
    fn set_slice(&self, low: usize, high: usize, seq: &Bound<'_, PyAny>) -> PyResult<()>;

    /// Deletes the slice from `low` to `high` from `self`.
    ///
    /// This is equivalent to the Python statement `del self[low:high]`.
    fn del_slice(&self, low: usize, high: usize) -> PyResult<()>;

    /// Appends an item to the list.
    fn append<I>(&self, item: I) -> PyResult<()>
    where
        I: ToPyObject;

    /// Inserts an item at the specified index.
    ///
    /// If `index >= self.len()`, inserts at the end.
    fn insert<I>(&self, index: usize, item: I) -> PyResult<()>
    where
        I: ToPyObject;

    /// Determines if self contains `value`.
    ///
    /// This is equivalent to the Python expression `value in self`.
    fn contains<V>(&self, value: V) -> PyResult<bool>
    where
        V: ToPyObject;

    /// Returns the first index `i` for which `self[i] == value`.
    ///
    /// This is equivalent to the Python expression `self.index(value)`.
    fn index<V>(&self, value: V) -> PyResult<usize>
    where
        V: ToPyObject;

    /// Returns an iterator over this list's items.
    fn iter(&self) -> BoundListIterator<'py>;

    /// Sorts the list in-place. Equivalent to the Python expression `l.sort()`.
    fn sort(&self) -> PyResult<()>;

    /// Reverses the list in-place. Equivalent to the Python expression `l.reverse()`.
    fn reverse(&self) -> PyResult<()>;

    /// Return a new tuple containing the contents of the list; equivalent to the Python expression `tuple(list)`.
    ///
    /// This method is equivalent to `self.as_sequence().to_tuple()` and faster than `PyTuple::new(py, this_list)`.
    fn to_tuple(&self) -> Bound<'py, PyTuple>;
}

impl<'py> PyListMethods<'py> for Bound<'py, PyList> {
    /// Returns the length of the list.
    fn len(&self) -> usize {
        unsafe {
            #[cfg(not(Py_LIMITED_API))]
            let size = ffi::PyList_GET_SIZE(self.as_ptr());
            #[cfg(Py_LIMITED_API)]
            let size = ffi::PyList_Size(self.as_ptr());

            // non-negative Py_ssize_t should always fit into Rust usize
            size as usize
        }
    }

    /// Checks if the list is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns `self` cast as a `PySequence`.
    fn as_sequence(&self) -> &Bound<'py, PySequence> {
        unsafe { self.downcast_unchecked() }
    }

    /// Returns `self` cast as a `PySequence`.
    fn into_sequence(self) -> Bound<'py, PySequence> {
        unsafe { self.into_any().downcast_into_unchecked() }
    }

    /// Gets the list item at the specified index.
    /// # Example
    /// ```
    /// use pyo3::{prelude::*, types::PyList};
    /// Python::with_gil(|py| {
    ///     let list = PyList::new_bound(py, [2, 3, 5, 7]);
    ///     let obj = list.get_item(0);
    ///     assert_eq!(obj.unwrap().extract::<i32>().unwrap(), 2);
    /// });
    /// ```
    fn get_item(&self, index: usize) -> PyResult<Bound<'py, PyAny>> {
        unsafe {
            // PyList_GetItem return borrowed ptr; must make owned for safety (see #890).
            ffi::PyList_GetItem(self.as_ptr(), index as Py_ssize_t)
                .assume_borrowed_or_err(self.py())
                .map(Borrowed::to_owned)
        }
    }

    /// Gets the list item at the specified index. Undefined behavior on bad index. Use with caution.
    ///
    /// # Safety
    ///
    /// Caller must verify that the index is within the bounds of the list.
    #[cfg(not(Py_LIMITED_API))]
    unsafe fn get_item_unchecked(&self, index: usize) -> Bound<'py, PyAny> {
        // PyList_GET_ITEM return borrowed ptr; must make owned for safety (see #890).
        ffi::PyList_GET_ITEM(self.as_ptr(), index as Py_ssize_t)
            .assume_borrowed(self.py())
            .to_owned()
    }

    /// Takes the slice `self[low:high]` and returns it as a new list.
    ///
    /// Indices must be nonnegative, and out-of-range indices are clipped to
    /// `self.len()`.
    fn get_slice(&self, low: usize, high: usize) -> Bound<'py, PyList> {
        unsafe {
            ffi::PyList_GetSlice(self.as_ptr(), get_ssize_index(low), get_ssize_index(high))
                .assume_owned(self.py())
                .downcast_into_unchecked()
        }
    }

    /// Sets the item at the specified index.
    ///
    /// Raises `IndexError` if the index is out of range.
    fn set_item<I>(&self, index: usize, item: I) -> PyResult<()>
    where
        I: ToPyObject,
    {
        fn inner(list: &Bound<'_, PyList>, index: usize, item: Bound<'_, PyAny>) -> PyResult<()> {
            err::error_on_minusone(list.py(), unsafe {
                ffi::PyList_SetItem(list.as_ptr(), get_ssize_index(index), item.into_ptr())
            })
        }

        let py = self.py();
        inner(self, index, item.to_object(py).into_bound(py))
    }

    /// Deletes the `index`th element of self.
    ///
    /// This is equivalent to the Python statement `del self[i]`.
    #[inline]
    fn del_item(&self, index: usize) -> PyResult<()> {
        self.as_sequence().del_item(index)
    }

    /// Assigns the sequence `seq` to the slice of `self` from `low` to `high`.
    ///
    /// This is equivalent to the Python statement `self[low:high] = v`.
    #[inline]
    fn set_slice(&self, low: usize, high: usize, seq: &Bound<'_, PyAny>) -> PyResult<()> {
        err::error_on_minusone(self.py(), unsafe {
            ffi::PyList_SetSlice(
                self.as_ptr(),
                get_ssize_index(low),
                get_ssize_index(high),
                seq.as_ptr(),
            )
        })
    }

    /// Deletes the slice from `low` to `high` from `self`.
    ///
    /// This is equivalent to the Python statement `del self[low:high]`.
    #[inline]
    fn del_slice(&self, low: usize, high: usize) -> PyResult<()> {
        self.as_sequence().del_slice(low, high)
    }

    /// Appends an item to the list.
    fn append<I>(&self, item: I) -> PyResult<()>
    where
        I: ToPyObject,
    {
        fn inner(list: &Bound<'_, PyList>, item: Bound<'_, PyAny>) -> PyResult<()> {
            err::error_on_minusone(list.py(), unsafe {
                ffi::PyList_Append(list.as_ptr(), item.as_ptr())
            })
        }

        let py = self.py();
        inner(self, item.to_object(py).into_bound(py))
    }

    /// Inserts an item at the specified index.
    ///
    /// If `index >= self.len()`, inserts at the end.
    fn insert<I>(&self, index: usize, item: I) -> PyResult<()>
    where
        I: ToPyObject,
    {
        fn inner(list: &Bound<'_, PyList>, index: usize, item: Bound<'_, PyAny>) -> PyResult<()> {
            err::error_on_minusone(list.py(), unsafe {
                ffi::PyList_Insert(list.as_ptr(), get_ssize_index(index), item.as_ptr())
            })
        }

        let py = self.py();
        inner(self, index, item.to_object(py).into_bound(py))
    }

    /// Determines if self contains `value`.
    ///
    /// This is equivalent to the Python expression `value in self`.
    #[inline]
    fn contains<V>(&self, value: V) -> PyResult<bool>
    where
        V: ToPyObject,
    {
        self.as_sequence().contains(value)
    }

    /// Returns the first index `i` for which `self[i] == value`.
    ///
    /// This is equivalent to the Python expression `self.index(value)`.
    #[inline]
    fn index<V>(&self, value: V) -> PyResult<usize>
    where
        V: ToPyObject,
    {
        self.as_sequence().index(value)
    }

    /// Returns an iterator over this list's items.
    fn iter(&self) -> BoundListIterator<'py> {
        BoundListIterator::new(self.clone())
    }

    /// Sorts the list in-place. Equivalent to the Python expression `l.sort()`.
    fn sort(&self) -> PyResult<()> {
        err::error_on_minusone(self.py(), unsafe { ffi::PyList_Sort(self.as_ptr()) })
    }

    /// Reverses the list in-place. Equivalent to the Python expression `l.reverse()`.
    fn reverse(&self) -> PyResult<()> {
        err::error_on_minusone(self.py(), unsafe { ffi::PyList_Reverse(self.as_ptr()) })
    }

    /// Return a new tuple containing the contents of the list; equivalent to the Python expression `tuple(list)`.
    ///
    /// This method is equivalent to `self.as_sequence().to_tuple()` and faster than `PyTuple::new(py, this_list)`.
    fn to_tuple(&self) -> Bound<'py, PyTuple> {
        unsafe {
            ffi::PyList_AsTuple(self.as_ptr())
                .assume_owned(self.py())
                .downcast_into_unchecked()
        }
    }
}

/// Used by `PyList::iter()`.
pub struct PyListIterator<'a>(BoundListIterator<'a>);

impl<'a> Iterator for PyListIterator<'a> {
    type Item = &'a PyAny;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(Bound::into_gil_ref)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a> DoubleEndedIterator for PyListIterator<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(Bound::into_gil_ref)
    }
}

impl<'a> ExactSizeIterator for PyListIterator<'a> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl FusedIterator for PyListIterator<'_> {}

impl<'a> IntoIterator for &'a PyList {
    type Item = &'a PyAny;
    type IntoIter = PyListIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Used by `PyList::iter()`.
pub struct BoundListIterator<'py> {
    list: Bound<'py, PyList>,
    index: usize,
    length: usize,
}

impl<'py> BoundListIterator<'py> {
    fn new(list: Bound<'py, PyList>) -> Self {
        let length: usize = list.len();
        BoundListIterator {
            list,
            index: 0,
            length,
        }
    }

    unsafe fn get_item(&self, index: usize) -> Bound<'py, PyAny> {
        #[cfg(any(Py_LIMITED_API, PyPy))]
        let item = self.list.get_item(index).expect("list.get failed");
        #[cfg(not(any(Py_LIMITED_API, PyPy)))]
        let item = self.list.get_item_unchecked(index);
        item
    }
}

impl<'py> Iterator for BoundListIterator<'py> {
    type Item = Bound<'py, PyAny>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let length = self.length.min(self.list.len());

        if self.index < length {
            let item = unsafe { self.get_item(self.index) };
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl DoubleEndedIterator for BoundListIterator<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let length = self.length.min(self.list.len());

        if self.index < length {
            let item = unsafe { self.get_item(length - 1) };
            self.length = length - 1;
            Some(item)
        } else {
            None
        }
    }
}

impl ExactSizeIterator for BoundListIterator<'_> {
    fn len(&self) -> usize {
        self.length.saturating_sub(self.index)
    }
}

impl FusedIterator for BoundListIterator<'_> {}

impl<'py> IntoIterator for Bound<'py, PyList> {
    type Item = Bound<'py, PyAny>;
    type IntoIter = BoundListIterator<'py>;

    fn into_iter(self) -> Self::IntoIter {
        BoundListIterator::new(self)
    }
}

impl<'py> IntoIterator for &Bound<'py, PyList> {
    type Item = Bound<'py, PyAny>;
    type IntoIter = BoundListIterator<'py>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
#[cfg_attr(not(feature = "gil-refs"), allow(deprecated))]
mod tests {
    use crate::types::any::PyAnyMethods;
    use crate::types::list::PyListMethods;
    use crate::types::sequence::PySequenceMethods;
    use crate::types::{PyList, PyTuple};
    use crate::Python;
    use crate::{IntoPy, PyObject, ToPyObject};

    #[test]
    fn test_new() {
        Python::with_gil(|py| {
            let list = PyList::new(py, [2, 3, 5, 7]);
            assert_eq!(2, list[0].extract::<i32>().unwrap());
            assert_eq!(3, list[1].extract::<i32>().unwrap());
            assert_eq!(5, list[2].extract::<i32>().unwrap());
            assert_eq!(7, list[3].extract::<i32>().unwrap());
        });
    }

    #[test]
    fn test_len() {
        Python::with_gil(|py| {
            let list = PyList::new(py, [1, 2, 3, 4]);
            assert_eq!(4, list.len());
        });
    }

    #[test]
    fn test_get_item() {
        Python::with_gil(|py| {
            let list = PyList::new(py, [2, 3, 5, 7]);
            assert_eq!(2, list.get_item(0).unwrap().extract::<i32>().unwrap());
            assert_eq!(3, list.get_item(1).unwrap().extract::<i32>().unwrap());
            assert_eq!(5, list.get_item(2).unwrap().extract::<i32>().unwrap());
            assert_eq!(7, list.get_item(3).unwrap().extract::<i32>().unwrap());
        });
    }

    #[test]
    fn test_get_slice() {
        Python::with_gil(|py| {
            let list = PyList::new(py, [2, 3, 5, 7]);
            let slice = list.get_slice(1, 3);
            assert_eq!(2, slice.len());
            let slice = list.get_slice(1, 7);
            assert_eq!(3, slice.len());
        });
    }

    #[test]
    fn test_set_item() {
        Python::with_gil(|py| {
            let list = PyList::new(py, [2, 3, 5, 7]);
            let val = 42i32.to_object(py);
            let val2 = 42i32.to_object(py);
            assert_eq!(2, list[0].extract::<i32>().unwrap());
            list.set_item(0, val).unwrap();
            assert_eq!(42, list[0].extract::<i32>().unwrap());
            assert!(list.set_item(10, val2).is_err());
        });
    }

    #[test]
    fn test_set_item_refcnt() {
        Python::with_gil(|py| {
            let obj = py.eval("object()", None, None).unwrap();
            let cnt;
            {
                let _pool = unsafe { crate::GILPool::new() };
                let v = vec![2];
                let ob = v.to_object(py);
                let list: &PyList = ob.downcast(py).unwrap();
                cnt = obj.get_refcnt();
                list.set_item(0, obj).unwrap();
            }

            assert_eq!(cnt, obj.get_refcnt());
        });
    }

    #[test]
    fn test_insert() {
        Python::with_gil(|py| {
            let list = PyList::new(py, [2, 3, 5, 7]);
            let val = 42i32.to_object(py);
            let val2 = 43i32.to_object(py);
            assert_eq!(4, list.len());
            assert_eq!(2, list[0].extract::<i32>().unwrap());
            list.insert(0, val).unwrap();
            list.insert(1000, val2).unwrap();
            assert_eq!(6, list.len());
            assert_eq!(42, list[0].extract::<i32>().unwrap());
            assert_eq!(2, list[1].extract::<i32>().unwrap());
            assert_eq!(43, list[5].extract::<i32>().unwrap());
        });
    }

    #[test]
    fn test_insert_refcnt() {
        Python::with_gil(|py| {
            let cnt;
            let obj = py.eval("object()", None, None).unwrap();
            {
                let _pool = unsafe { crate::GILPool::new() };
                let list = PyList::empty(py);
                cnt = obj.get_refcnt();
                list.insert(0, obj).unwrap();
            }

            assert_eq!(cnt, obj.get_refcnt());
        });
    }

    #[test]
    fn test_append() {
        Python::with_gil(|py| {
            let list = PyList::new(py, [2]);
            list.append(3).unwrap();
            assert_eq!(2, list[0].extract::<i32>().unwrap());
            assert_eq!(3, list[1].extract::<i32>().unwrap());
        });
    }

    #[test]
    fn test_append_refcnt() {
        Python::with_gil(|py| {
            let cnt;
            let obj = py.eval("object()", None, None).unwrap();
            {
                let _pool = unsafe { crate::GILPool::new() };
                let list = PyList::empty(py);
                cnt = obj.get_refcnt();
                list.append(obj).unwrap();
            }
            assert_eq!(cnt, obj.get_refcnt());
        });
    }

    #[test]
    fn test_iter() {
        Python::with_gil(|py| {
            let v = vec![2, 3, 5, 7];
            let list = PyList::new(py, &v);
            let mut idx = 0;
            for el in list {
                assert_eq!(v[idx], el.extract::<i32>().unwrap());
                idx += 1;
            }
            assert_eq!(idx, v.len());
        });
    }

    #[test]
    fn test_iter_size_hint() {
        Python::with_gil(|py| {
            let v = vec![2, 3, 5, 7];
            let ob = v.to_object(py);
            let list: &PyList = ob.downcast(py).unwrap();

            let mut iter = list.iter();
            assert_eq!(iter.size_hint(), (v.len(), Some(v.len())));
            iter.next();
            assert_eq!(iter.size_hint(), (v.len() - 1, Some(v.len() - 1)));

            // Exhaust iterator.
            for _ in &mut iter {}

            assert_eq!(iter.size_hint(), (0, Some(0)));
        });
    }

    #[test]
    fn test_iter_rev() {
        Python::with_gil(|py| {
            let v = vec![2, 3, 5, 7];
            let ob = v.to_object(py);
            let list: &PyList = ob.downcast(py).unwrap();

            let mut iter = list.iter().rev();

            assert_eq!(iter.size_hint(), (4, Some(4)));

            assert_eq!(iter.next().unwrap().extract::<i32>().unwrap(), 7);
            assert_eq!(iter.size_hint(), (3, Some(3)));

            assert_eq!(iter.next().unwrap().extract::<i32>().unwrap(), 5);
            assert_eq!(iter.size_hint(), (2, Some(2)));

            assert_eq!(iter.next().unwrap().extract::<i32>().unwrap(), 3);
            assert_eq!(iter.size_hint(), (1, Some(1)));

            assert_eq!(iter.next().unwrap().extract::<i32>().unwrap(), 2);
            assert_eq!(iter.size_hint(), (0, Some(0)));

            assert!(iter.next().is_none());
            assert!(iter.next().is_none());
        });
    }

    #[test]
    fn test_into_iter() {
        Python::with_gil(|py| {
            let list = PyList::new(py, [1, 2, 3, 4]);
            for (i, item) in list.iter().enumerate() {
                assert_eq!((i + 1) as i32, item.extract::<i32>().unwrap());
            }
        });
    }

    #[test]
    fn test_into_iter_bound() {
        use crate::types::any::PyAnyMethods;

        Python::with_gil(|py| {
            let list = PyList::new_bound(py, [1, 2, 3, 4]);
            let mut items = vec![];
            for item in &list {
                items.push(item.extract::<i32>().unwrap());
            }
            assert_eq!(items, vec![1, 2, 3, 4]);
        });
    }

    #[test]
    fn test_as_sequence() {
        Python::with_gil(|py| {
            let list = PyList::new_bound(py, [1, 2, 3, 4]);

            assert_eq!(list.as_sequence().len().unwrap(), 4);
            assert_eq!(
                list.as_sequence()
                    .get_item(1)
                    .unwrap()
                    .extract::<i32>()
                    .unwrap(),
                2
            );
        });
    }

    #[test]
    fn test_into_sequence() {
        Python::with_gil(|py| {
            let list = PyList::new_bound(py, [1, 2, 3, 4]);

            let sequence = list.into_sequence();

            assert_eq!(sequence.len().unwrap(), 4);
            assert_eq!(sequence.get_item(1).unwrap().extract::<i32>().unwrap(), 2);
        });
    }

    #[test]
    fn test_extract() {
        Python::with_gil(|py| {
            let v = vec![2, 3, 5, 7];
            let list = PyList::new(py, &v);
            let v2 = list.as_ref().extract::<Vec<i32>>().unwrap();
            assert_eq!(v, v2);
        });
    }

    #[test]
    fn test_sort() {
        Python::with_gil(|py| {
            let v = vec![7, 3, 2, 5];
            let list = PyList::new(py, &v);
            assert_eq!(7, list[0].extract::<i32>().unwrap());
            assert_eq!(3, list[1].extract::<i32>().unwrap());
            assert_eq!(2, list[2].extract::<i32>().unwrap());
            assert_eq!(5, list[3].extract::<i32>().unwrap());
            list.sort().unwrap();
            assert_eq!(2, list[0].extract::<i32>().unwrap());
            assert_eq!(3, list[1].extract::<i32>().unwrap());
            assert_eq!(5, list[2].extract::<i32>().unwrap());
            assert_eq!(7, list[3].extract::<i32>().unwrap());
        });
    }

    #[test]
    fn test_reverse() {
        Python::with_gil(|py| {
            let v = vec![2, 3, 5, 7];
            let list = PyList::new(py, &v);
            assert_eq!(2, list[0].extract::<i32>().unwrap());
            assert_eq!(3, list[1].extract::<i32>().unwrap());
            assert_eq!(5, list[2].extract::<i32>().unwrap());
            assert_eq!(7, list[3].extract::<i32>().unwrap());
            list.reverse().unwrap();
            assert_eq!(7, list[0].extract::<i32>().unwrap());
            assert_eq!(5, list[1].extract::<i32>().unwrap());
            assert_eq!(3, list[2].extract::<i32>().unwrap());
            assert_eq!(2, list[3].extract::<i32>().unwrap());
        });
    }

    #[test]
    fn test_array_into_py() {
        Python::with_gil(|py| {
            let array: PyObject = [1, 2].into_py(py);
            let list: &PyList = array.downcast(py).unwrap();
            assert_eq!(1, list[0].extract::<i32>().unwrap());
            assert_eq!(2, list[1].extract::<i32>().unwrap());
        });
    }

    #[test]
    fn test_list_get_item_invalid_index() {
        Python::with_gil(|py| {
            let list = PyList::new(py, [2, 3, 5, 7]);
            let obj = list.get_item(5);
            assert!(obj.is_err());
            assert_eq!(
                obj.unwrap_err().to_string(),
                "IndexError: list index out of range"
            );
        });
    }

    #[test]
    fn test_list_get_item_sanity() {
        Python::with_gil(|py| {
            let list = PyList::new(py, [2, 3, 5, 7]);
            let obj = list.get_item(0);
            assert_eq!(obj.unwrap().extract::<i32>().unwrap(), 2);
        });
    }

    #[cfg(not(any(Py_LIMITED_API, PyPy)))]
    #[test]
    fn test_list_get_item_unchecked_sanity() {
        Python::with_gil(|py| {
            let list = PyList::new(py, [2, 3, 5, 7]);
            let obj = unsafe { list.get_item_unchecked(0) };
            assert_eq!(obj.extract::<i32>().unwrap(), 2);
        });
    }

    #[test]
    fn test_list_index_trait() {
        Python::with_gil(|py| {
            let list = PyList::new(py, [2, 3, 5]);
            assert_eq!(2, list[0].extract::<i32>().unwrap());
            assert_eq!(3, list[1].extract::<i32>().unwrap());
            assert_eq!(5, list[2].extract::<i32>().unwrap());
        });
    }

    #[test]
    #[should_panic]
    fn test_list_index_trait_panic() {
        Python::with_gil(|py| {
            let list = PyList::new(py, [2, 3, 5]);
            let _ = &list[7];
        });
    }

    #[test]
    fn test_list_index_trait_ranges() {
        Python::with_gil(|py| {
            let list = PyList::new(py, [2, 3, 5]);
            assert_eq!(vec![3, 5], list[1..3].extract::<Vec<i32>>().unwrap());
            assert_eq!(Vec::<i32>::new(), list[3..3].extract::<Vec<i32>>().unwrap());
            assert_eq!(vec![3, 5], list[1..].extract::<Vec<i32>>().unwrap());
            assert_eq!(Vec::<i32>::new(), list[3..].extract::<Vec<i32>>().unwrap());
            assert_eq!(vec![2, 3, 5], list[..].extract::<Vec<i32>>().unwrap());
            assert_eq!(vec![3, 5], list[1..=2].extract::<Vec<i32>>().unwrap());
            assert_eq!(vec![2, 3], list[..2].extract::<Vec<i32>>().unwrap());
            assert_eq!(vec![2, 3], list[..=1].extract::<Vec<i32>>().unwrap());
        })
    }

    #[test]
    #[should_panic = "range start index 5 out of range for list of length 3"]
    fn test_list_index_trait_range_panic_start() {
        Python::with_gil(|py| {
            let list = PyList::new(py, [2, 3, 5]);
            list[5..10].extract::<Vec<i32>>().unwrap();
        })
    }

    #[test]
    #[should_panic = "range end index 10 out of range for list of length 3"]
    fn test_list_index_trait_range_panic_end() {
        Python::with_gil(|py| {
            let list = PyList::new(py, [2, 3, 5]);
            list[1..10].extract::<Vec<i32>>().unwrap();
        })
    }

    #[test]
    #[should_panic = "slice index starts at 2 but ends at 1"]
    fn test_list_index_trait_range_panic_wrong_order() {
        Python::with_gil(|py| {
            let list = PyList::new(py, [2, 3, 5]);
            #[allow(clippy::reversed_empty_ranges)]
            list[2..1].extract::<Vec<i32>>().unwrap();
        })
    }

    #[test]
    #[should_panic = "range start index 8 out of range for list of length 3"]
    fn test_list_index_trait_range_from_panic() {
        Python::with_gil(|py| {
            let list = PyList::new(py, [2, 3, 5]);
            list[8..].extract::<Vec<i32>>().unwrap();
        })
    }

    #[test]
    fn test_list_del_item() {
        Python::with_gil(|py| {
            let list = PyList::new(py, [1, 1, 2, 3, 5, 8]);
            assert!(list.del_item(10).is_err());
            assert_eq!(1, list[0].extract::<i32>().unwrap());
            assert!(list.del_item(0).is_ok());
            assert_eq!(1, list[0].extract::<i32>().unwrap());
            assert!(list.del_item(0).is_ok());
            assert_eq!(2, list[0].extract::<i32>().unwrap());
            assert!(list.del_item(0).is_ok());
            assert_eq!(3, list[0].extract::<i32>().unwrap());
            assert!(list.del_item(0).is_ok());
            assert_eq!(5, list[0].extract::<i32>().unwrap());
            assert!(list.del_item(0).is_ok());
            assert_eq!(8, list[0].extract::<i32>().unwrap());
            assert!(list.del_item(0).is_ok());
            assert_eq!(0, list.len());
            assert!(list.del_item(0).is_err());
        });
    }

    #[test]
    fn test_list_set_slice() {
        Python::with_gil(|py| {
            let list = PyList::new(py, [1, 1, 2, 3, 5, 8]);
            let ins = PyList::new(py, [7, 4]);
            list.set_slice(1, 4, ins).unwrap();
            assert_eq!([1, 7, 4, 5, 8], list.extract::<[i32; 5]>().unwrap());
            list.set_slice(3, 100, PyList::empty(py)).unwrap();
            assert_eq!([1, 7, 4], list.extract::<[i32; 3]>().unwrap());
        });
    }

    #[test]
    fn test_list_del_slice() {
        Python::with_gil(|py| {
            let list = PyList::new(py, [1, 1, 2, 3, 5, 8]);
            list.del_slice(1, 4).unwrap();
            assert_eq!([1, 5, 8], list.extract::<[i32; 3]>().unwrap());
            list.del_slice(1, 100).unwrap();
            assert_eq!([1], list.extract::<[i32; 1]>().unwrap());
        });
    }

    #[test]
    fn test_list_contains() {
        Python::with_gil(|py| {
            let list = PyList::new(py, [1, 1, 2, 3, 5, 8]);
            assert_eq!(6, list.len());

            let bad_needle = 7i32.to_object(py);
            assert!(!list.contains(&bad_needle).unwrap());

            let good_needle = 8i32.to_object(py);
            assert!(list.contains(&good_needle).unwrap());

            let type_coerced_needle = 8f32.to_object(py);
            assert!(list.contains(&type_coerced_needle).unwrap());
        });
    }

    #[test]
    fn test_list_index() {
        Python::with_gil(|py| {
            let list = PyList::new(py, [1, 1, 2, 3, 5, 8]);
            assert_eq!(0, list.index(1i32).unwrap());
            assert_eq!(2, list.index(2i32).unwrap());
            assert_eq!(3, list.index(3i32).unwrap());
            assert_eq!(4, list.index(5i32).unwrap());
            assert_eq!(5, list.index(8i32).unwrap());
            assert!(list.index(42i32).is_err());
        });
    }

    use std::ops::Range;

    // An iterator that lies about its `ExactSizeIterator` implementation.
    // See https://github.com/PyO3/pyo3/issues/2118
    struct FaultyIter(Range<usize>, usize);

    impl Iterator for FaultyIter {
        type Item = usize;

        fn next(&mut self) -> Option<Self::Item> {
            self.0.next()
        }
    }

    impl ExactSizeIterator for FaultyIter {
        fn len(&self) -> usize {
            self.1
        }
    }

    #[test]
    #[should_panic(
        expected = "Attempted to create PyList but `elements` was larger than reported by its `ExactSizeIterator` implementation."
    )]
    fn too_long_iterator() {
        Python::with_gil(|py| {
            let iter = FaultyIter(0..usize::MAX, 73);
            let _list = PyList::new(py, iter);
        })
    }

    #[test]
    #[should_panic(
        expected = "Attempted to create PyList but `elements` was smaller than reported by its `ExactSizeIterator` implementation."
    )]
    fn too_short_iterator() {
        Python::with_gil(|py| {
            let iter = FaultyIter(0..35, 73);
            let _list = PyList::new(py, iter);
        })
    }

    #[test]
    #[should_panic(
        expected = "out of range integral type conversion attempted on `elements.len()`"
    )]
    fn overflowing_size() {
        Python::with_gil(|py| {
            let iter = FaultyIter(0..0, usize::MAX);

            let _list = PyList::new(py, iter);
        })
    }

    #[cfg(feature = "macros")]
    #[test]
    fn bad_clone_mem_leaks() {
        use crate::{Py, PyAny};
        use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};
        static NEEDS_DESTRUCTING_COUNT: AtomicUsize = AtomicUsize::new(0);

        #[crate::pyclass]
        #[pyo3(crate = "crate")]
        struct Bad(usize);

        impl Clone for Bad {
            fn clone(&self) -> Self {
                // This panic should not lead to a memory leak
                assert_ne!(self.0, 42);
                NEEDS_DESTRUCTING_COUNT.fetch_add(1, SeqCst);

                Bad(self.0)
            }
        }

        impl Drop for Bad {
            fn drop(&mut self) {
                NEEDS_DESTRUCTING_COUNT.fetch_sub(1, SeqCst);
            }
        }

        impl ToPyObject for Bad {
            fn to_object(&self, py: Python<'_>) -> Py<PyAny> {
                self.to_owned().into_py(py)
            }
        }

        struct FaultyIter(Range<usize>, usize);

        impl Iterator for FaultyIter {
            type Item = Bad;

            fn next(&mut self) -> Option<Self::Item> {
                self.0.next().map(|i| {
                    NEEDS_DESTRUCTING_COUNT.fetch_add(1, SeqCst);
                    Bad(i)
                })
            }
        }

        impl ExactSizeIterator for FaultyIter {
            fn len(&self) -> usize {
                self.1
            }
        }

        Python::with_gil(|py| {
            std::panic::catch_unwind(|| {
                let iter = FaultyIter(0..50, 50);
                let _list = PyList::new(py, iter);
            })
            .unwrap_err();
        });

        assert_eq!(
            NEEDS_DESTRUCTING_COUNT.load(SeqCst),
            0,
            "Some destructors did not run"
        );
    }

    #[test]
    fn test_list_to_tuple() {
        Python::with_gil(|py| {
            let list = PyList::new(py, vec![1, 2, 3]);
            let tuple = list.to_tuple();
            let tuple_expected = PyTuple::new_bound(py, vec![1, 2, 3]);
            assert!(tuple.eq(tuple_expected).unwrap());
        })
    }
}
