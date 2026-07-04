use std::{any::TypeId, fmt::Debug, marker::PhantomData, ops::Deref};

use crate::injected::{Injectable, ValueInjected};

pub struct TypedValueInjected<T> {
    value: ValueInjected,
    _phantom: PhantomData<T>,
}

impl<T> TypedValueInjected<T> {
    /// Create a new typed value injected
    pub fn new(value: T) -> Self
    where
        T: Injectable,
    {
        Self {
            value: ValueInjected::new(value),
            _phantom: PhantomData,
        }
    }

    pub fn project<U>(self, fun: impl for<'a> FnOnce(&'a T) -> &'a U) -> TypedValueInjected<U>
    where
        U: Injectable,
        T: 'static,
    {
        let projected = self.value.0.map_project(|content, _| {
            let content = unsafe {
                // Safety: nobody has mutable access until the type is unwrapped, so
                // the type cannot have changed
                &*(content as *const dyn Injectable as *const T)
            };
            let project = fun(content);

            project as &dyn Injectable
        });

        TypedValueInjected {
            value: ValueInjected(projected),
            _phantom: PhantomData,
        }
    }

    /// Create a new typed value injected from a static value
    pub fn new_static(value: &'static T) -> Self
    where
        T: Injectable,
    {
        Self {
            value: ValueInjected::new_static(value),
            _phantom: PhantomData,
        }
    }

    /// Check the type of a injected value and downcast
    pub fn downcast(value: ValueInjected) -> Option<Self>
    where
        T: Injectable,
    {
        if value.0.get().type_id() != TypeId::of::<T>() {
            return None;
        }

        Some(Self {
            value,
            _phantom: PhantomData,
        })
    }

    /// Forget the type
    pub fn type_erase(Self { value, .. }: Self) -> ValueInjected {
        value
    }

    /// Forget the type
    pub fn as_type_erased(&self) -> &ValueInjected {
        &self.value
    }
}

impl<T> Deref for TypedValueInjected<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            // Safety: nobody has mutable access until the type is unwrapped, so
            // the type cannot have changed
            &*((*self.value.0.get()) as *const dyn Injectable as *const T)
        }
    }
}

impl<T> Debug for TypedValueInjected<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TypedValueInjected")
            .field("value", &self.value)
            .field("_phantom", &self._phantom)
            .finish()
    }
}

impl<T> Clone for TypedValueInjected<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            _phantom: self._phantom,
        }
    }
}
