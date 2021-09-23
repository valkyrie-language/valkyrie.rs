#[repr(u8)]
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AsynchronousKind {
    Auto,
    Asynchronous,
    Synchronous,
}
impl ::core::fmt::Debug for AsynchronousKind {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        match self {
            AsynchronousKind::Auto => f.debug_tuple("AsynchronousKind::Auto").finish(),
            AsynchronousKind::Asynchronous => f.debug_tuple("AsynchronousKind::Asynchronous").finish(),
            AsynchronousKind::Synchronous => f.debug_tuple("AsynchronousKind::Synchronous").finish(),
        }
    }
}
impl AsynchronousKind {
    #[doc(hidden)]
    pub unsafe fn _lift(val: u8) -> AsynchronousKind {
        if !cfg!(debug_assertions) {
            return ::core::mem::transmute(val);
        }
        match val {
            0 => AsynchronousKind::Auto,
            1 => AsynchronousKind::Asynchronous,
            2 => AsynchronousKind::Synchronous,
            _ => panic!("invalid enum discriminant"),
        }
    }
}
#[repr(u8)]
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum FunctionKind {
    /// A function that lazy evaluate the arguments
    ///
    /// `macro function(args)`
    Macro,
    /// A function that eager evaluates the arguments
    ///
    /// `micro function(args)`
    Micro,
}
impl ::core::fmt::Debug for FunctionKind {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        match self {
            FunctionKind::Macro => f.debug_tuple("FunctionKind::Macro").finish(),
            FunctionKind::Micro => f.debug_tuple("FunctionKind::Micro").finish(),
        }
    }
}
impl FunctionKind {
    #[doc(hidden)]
    pub unsafe fn _lift(val: u8) -> FunctionKind {
        if !cfg!(debug_assertions) {
            return ::core::mem::transmute(val);
        }
        match val {
            0 => FunctionKind::Macro,
            1 => FunctionKind::Micro,
            _ => panic!("invalid enum discriminant"),
        }
    }
}
#[cfg(target_arch = "wasm32")]
#[link_section = "component-type:wit-bindgen:0.35.0:valkyrie:ast:types:encoded world"]
#[doc(hidden)]
pub static __WIT_BINDGEN_COMPONENT_TYPE: [u8; 242] = *b"\
\0asm\x0d\0\x01\0\0\x19\x16wit-component-encoding\x04\0\x07w\x01A\x02\x01A\x04\x01\
m\x03\x04auto\x0casynchronous\x0bsynchronous\x03\0\x11asynchronous-kind\x03\0\0\x01\
m\x02\x05macro\x05micro\x03\0\x0dfunction-kind\x03\0\x02\x04\0\x12valkyrie:ast/t\
ypes\x04\0\x0b\x0b\x01\0\x05types\x03\0\0\0G\x09producers\x01\x0cprocessed-by\x02\
\x0dwit-component\x070.220.0\x10wit-bindgen-rust\x060.35.0";
#[inline(never)]
#[doc(hidden)]
pub fn __link_custom_section_describing_imports() {
    wit_bindgen_rt::maybe_link_cabi_realloc();
}
