#[derive(Clone)]
pub struct ToolsError {
    pub message: _rt::String,
}
impl ::core::fmt::Debug for ToolsError {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("ToolsError").field("message", &self.message).finish()
    }
}
impl ::core::fmt::Display for ToolsError {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for ToolsError {}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct EncodeConfig {
    pub generate_dwarf: bool,
}
impl ::core::fmt::Debug for EncodeConfig {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("EncodeConfig")
            .field("generate-dwarf", &self.generate_dwarf)
            .finish()
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct DecodeConfig {
    pub skeleton_only: bool,
    pub name_unnamed: bool,
    pub fold_instructions: bool,
}
impl ::core::fmt::Debug for DecodeConfig {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("DecodeConfig")
            .field("skeleton-only", &self.skeleton_only)
            .field("name-unnamed", &self.name_unnamed)
            .field("fold-instructions", &self.fold_instructions)
            .finish()
    }
}
#[derive(Clone)]
pub struct PolyfillConfig {
    pub name: _rt::String,
    pub shim: _rt::Vec<(_rt::String, _rt::String)>,
}
impl ::core::fmt::Debug for PolyfillConfig {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("PolyfillConfig")
            .field("name", &self.name)
            .field("shim", &self.shim)
            .finish()
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
pub unsafe fn _export_wat_encode_cabi<T: Guest>(
    arg0: *mut u8,
    arg1: usize,
    arg2: i32,
) -> *mut u8 {
    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
    let len0 = arg1;
    let bytes0 = _rt::Vec::from_raw_parts(arg0.cast(), len0, len0);
    let result1 = T::wat_encode(
        _rt::string_lift(bytes0),
        EncodeConfig {
            generate_dwarf: _rt::bool_lift(arg2 as u8),
        },
    );
    let ptr2 = _RET_AREA.0.as_mut_ptr().cast::<u8>();
    match result1 {
        Ok(e) => {
            *ptr2.add(0).cast::<u8>() = (0i32) as u8;
            let vec3 = (e).into_boxed_slice();
            let ptr3 = vec3.as_ptr().cast::<u8>();
            let len3 = vec3.len();
            ::core::mem::forget(vec3);
            *ptr2.add(8).cast::<usize>() = len3;
            *ptr2.add(4).cast::<*mut u8>() = ptr3.cast_mut();
        }
        Err(e) => {
            *ptr2.add(0).cast::<u8>() = (1i32) as u8;
            let ToolsError { message: message4 } = e;
            let vec5 = (message4.into_bytes()).into_boxed_slice();
            let ptr5 = vec5.as_ptr().cast::<u8>();
            let len5 = vec5.len();
            ::core::mem::forget(vec5);
            *ptr2.add(8).cast::<usize>() = len5;
            *ptr2.add(4).cast::<*mut u8>() = ptr5.cast_mut();
        }
    };
    ptr2
}
#[doc(hidden)]
#[allow(non_snake_case)]
pub unsafe fn __post_return_wat_encode<T: Guest>(arg0: *mut u8) {
    let l0 = i32::from(*arg0.add(0).cast::<u8>());
    match l0 {
        0 => {
            let l1 = *arg0.add(4).cast::<*mut u8>();
            let l2 = *arg0.add(8).cast::<usize>();
            let base3 = l1;
            let len3 = l2;
            _rt::cabi_dealloc(base3, len3 * 1, 1);
        }
        _ => {
            let l4 = *arg0.add(4).cast::<*mut u8>();
            let l5 = *arg0.add(8).cast::<usize>();
            _rt::cabi_dealloc(l4, l5, 1);
        }
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
pub unsafe fn _export_wasm_decode_cabi<T: Guest>(
    arg0: *mut u8,
    arg1: usize,
    arg2: i32,
    arg3: i32,
    arg4: i32,
) -> *mut u8 {
    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
    let len0 = arg1;
    let result1 = T::wasm_decode(
        _rt::Vec::from_raw_parts(arg0.cast(), len0, len0),
        DecodeConfig {
            skeleton_only: _rt::bool_lift(arg2 as u8),
            name_unnamed: _rt::bool_lift(arg3 as u8),
            fold_instructions: _rt::bool_lift(arg4 as u8),
        },
    );
    let ptr2 = _RET_AREA.0.as_mut_ptr().cast::<u8>();
    match result1 {
        Ok(e) => {
            *ptr2.add(0).cast::<u8>() = (0i32) as u8;
            let vec3 = (e.into_bytes()).into_boxed_slice();
            let ptr3 = vec3.as_ptr().cast::<u8>();
            let len3 = vec3.len();
            ::core::mem::forget(vec3);
            *ptr2.add(8).cast::<usize>() = len3;
            *ptr2.add(4).cast::<*mut u8>() = ptr3.cast_mut();
        }
        Err(e) => {
            *ptr2.add(0).cast::<u8>() = (1i32) as u8;
            let ToolsError { message: message4 } = e;
            let vec5 = (message4.into_bytes()).into_boxed_slice();
            let ptr5 = vec5.as_ptr().cast::<u8>();
            let len5 = vec5.len();
            ::core::mem::forget(vec5);
            *ptr2.add(8).cast::<usize>() = len5;
            *ptr2.add(4).cast::<*mut u8>() = ptr5.cast_mut();
        }
    };
    ptr2
}
#[doc(hidden)]
#[allow(non_snake_case)]
pub unsafe fn __post_return_wasm_decode<T: Guest>(arg0: *mut u8) {
    let l0 = i32::from(*arg0.add(0).cast::<u8>());
    match l0 {
        0 => {
            let l1 = *arg0.add(4).cast::<*mut u8>();
            let l2 = *arg0.add(8).cast::<usize>();
            _rt::cabi_dealloc(l1, l2, 1);
        }
        _ => {
            let l3 = *arg0.add(4).cast::<*mut u8>();
            let l4 = *arg0.add(8).cast::<usize>();
            _rt::cabi_dealloc(l3, l4, 1);
        }
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
pub unsafe fn _export_wasi_polyfill_cabi<T: Guest>(
    arg0: *mut u8,
    arg1: usize,
    arg2: *mut u8,
    arg3: usize,
    arg4: *mut u8,
    arg5: usize,
) -> *mut u8 {
    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
    let len0 = arg1;
    let len1 = arg3;
    let bytes1 = _rt::Vec::from_raw_parts(arg2.cast(), len1, len1);
    let base8 = arg4;
    let len8 = arg5;
    let mut result8 = _rt::Vec::with_capacity(len8);
    for i in 0..len8 {
        let base = base8.add(i * 16);
        let e8 = {
            let l2 = *base.add(0).cast::<*mut u8>();
            let l3 = *base.add(4).cast::<usize>();
            let len4 = l3;
            let bytes4 = _rt::Vec::from_raw_parts(l2.cast(), len4, len4);
            let l5 = *base.add(8).cast::<*mut u8>();
            let l6 = *base.add(12).cast::<usize>();
            let len7 = l6;
            let bytes7 = _rt::Vec::from_raw_parts(l5.cast(), len7, len7);
            (_rt::string_lift(bytes4), _rt::string_lift(bytes7))
        };
        result8.push(e8);
    }
    _rt::cabi_dealloc(base8, len8 * 16, 4);
    let result9 = T::wasi_polyfill(
        _rt::Vec::from_raw_parts(arg0.cast(), len0, len0),
        PolyfillConfig {
            name: _rt::string_lift(bytes1),
            shim: result8,
        },
    );
    let ptr10 = _RET_AREA.0.as_mut_ptr().cast::<u8>();
    match result9 {
        Ok(e) => {
            *ptr10.add(0).cast::<u8>() = (0i32) as u8;
            let vec14 = e;
            let len14 = vec14.len();
            let layout14 = _rt::alloc::Layout::from_size_align_unchecked(
                vec14.len() * 16,
                4,
            );
            let result14 = if layout14.size() != 0 {
                let ptr = _rt::alloc::alloc(layout14).cast::<u8>();
                if ptr.is_null() {
                    _rt::alloc::handle_alloc_error(layout14);
                }
                ptr
            } else {
                ::core::ptr::null_mut()
            };
            for (i, e) in vec14.into_iter().enumerate() {
                let base = result14.add(i * 16);
                {
                    let (t11_0, t11_1) = e;
                    let vec12 = (t11_0.into_bytes()).into_boxed_slice();
                    let ptr12 = vec12.as_ptr().cast::<u8>();
                    let len12 = vec12.len();
                    ::core::mem::forget(vec12);
                    *base.add(4).cast::<usize>() = len12;
                    *base.add(0).cast::<*mut u8>() = ptr12.cast_mut();
                    let vec13 = (t11_1).into_boxed_slice();
                    let ptr13 = vec13.as_ptr().cast::<u8>();
                    let len13 = vec13.len();
                    ::core::mem::forget(vec13);
                    *base.add(12).cast::<usize>() = len13;
                    *base.add(8).cast::<*mut u8>() = ptr13.cast_mut();
                }
            }
            *ptr10.add(8).cast::<usize>() = len14;
            *ptr10.add(4).cast::<*mut u8>() = result14;
        }
        Err(e) => {
            *ptr10.add(0).cast::<u8>() = (1i32) as u8;
            let ToolsError { message: message15 } = e;
            let vec16 = (message15.into_bytes()).into_boxed_slice();
            let ptr16 = vec16.as_ptr().cast::<u8>();
            let len16 = vec16.len();
            ::core::mem::forget(vec16);
            *ptr10.add(8).cast::<usize>() = len16;
            *ptr10.add(4).cast::<*mut u8>() = ptr16.cast_mut();
        }
    };
    ptr10
}
#[doc(hidden)]
#[allow(non_snake_case)]
pub unsafe fn __post_return_wasi_polyfill<T: Guest>(arg0: *mut u8) {
    let l0 = i32::from(*arg0.add(0).cast::<u8>());
    match l0 {
        0 => {
            let l1 = *arg0.add(4).cast::<*mut u8>();
            let l2 = *arg0.add(8).cast::<usize>();
            let base8 = l1;
            let len8 = l2;
            for i in 0..len8 {
                let base = base8.add(i * 16);
                {
                    let l3 = *base.add(0).cast::<*mut u8>();
                    let l4 = *base.add(4).cast::<usize>();
                    _rt::cabi_dealloc(l3, l4, 1);
                    let l5 = *base.add(8).cast::<*mut u8>();
                    let l6 = *base.add(12).cast::<usize>();
                    let base7 = l5;
                    let len7 = l6;
                    _rt::cabi_dealloc(base7, len7 * 1, 1);
                }
            }
            _rt::cabi_dealloc(base8, len8 * 16, 4);
        }
        _ => {
            let l9 = *arg0.add(4).cast::<*mut u8>();
            let l10 = *arg0.add(8).cast::<usize>();
            _rt::cabi_dealloc(l9, l10, 1);
        }
    }
}
pub trait Guest {
    fn wat_encode(
        input: _rt::String,
        config: EncodeConfig,
    ) -> Result<_rt::Vec<u8>, ToolsError>;
    fn wasm_decode(
        input: _rt::Vec<u8>,
        config: DecodeConfig,
    ) -> Result<_rt::String, ToolsError>;
    fn wasi_polyfill(
        input: _rt::Vec<u8>,
        config: PolyfillConfig,
    ) -> Result<_rt::Vec<(_rt::String, _rt::Vec<u8>)>, ToolsError>;
}
#[doc(hidden)]
macro_rules! __export_world_tools_cabi {
    ($ty:ident with_types_in $($path_to_types:tt)*) => {
        const _ : () = { #[export_name = "wat-encode"] unsafe extern "C" fn
        export_wat_encode(arg0 : * mut u8, arg1 : usize, arg2 : i32,) -> * mut u8 {
        $($path_to_types)*:: _export_wat_encode_cabi::<$ty > (arg0, arg1, arg2) }
        #[export_name = "cabi_post_wat-encode"] unsafe extern "C" fn
        _post_return_wat_encode(arg0 : * mut u8,) { $($path_to_types)*::
        __post_return_wat_encode::<$ty > (arg0) } #[export_name = "wasm-decode"] unsafe
        extern "C" fn export_wasm_decode(arg0 : * mut u8, arg1 : usize, arg2 : i32, arg3
        : i32, arg4 : i32,) -> * mut u8 { $($path_to_types)*::
        _export_wasm_decode_cabi::<$ty > (arg0, arg1, arg2, arg3, arg4) } #[export_name =
        "cabi_post_wasm-decode"] unsafe extern "C" fn _post_return_wasm_decode(arg0 : *
        mut u8,) { $($path_to_types)*:: __post_return_wasm_decode::<$ty > (arg0) }
        #[export_name = "wasi-polyfill"] unsafe extern "C" fn export_wasi_polyfill(arg0 :
        * mut u8, arg1 : usize, arg2 : * mut u8, arg3 : usize, arg4 : * mut u8, arg5 :
        usize,) -> * mut u8 { $($path_to_types)*:: _export_wasi_polyfill_cabi::<$ty >
        (arg0, arg1, arg2, arg3, arg4, arg5) } #[export_name = "cabi_post_wasi-polyfill"]
        unsafe extern "C" fn _post_return_wasi_polyfill(arg0 : * mut u8,) {
        $($path_to_types)*:: __post_return_wasi_polyfill::<$ty > (arg0) } };
    };
}
#[doc(hidden)]
pub(crate) use __export_world_tools_cabi;
#[repr(align(4))]
struct _RetArea([::core::mem::MaybeUninit<u8>; 12]);
static mut _RET_AREA: _RetArea = _RetArea([::core::mem::MaybeUninit::uninit(); 12]);
mod _rt {
    pub use alloc_crate::string::String;
    pub use alloc_crate::vec::Vec;
    #[cfg(target_arch = "wasm32")]
    pub fn run_ctors_once() {
        wit_bindgen_rt::run_ctors_once();
    }
    pub unsafe fn string_lift(bytes: Vec<u8>) -> String {
        if cfg!(debug_assertions) {
            String::from_utf8(bytes).unwrap()
        } else {
            String::from_utf8_unchecked(bytes)
        }
    }
    pub unsafe fn bool_lift(val: u8) -> bool {
        if cfg!(debug_assertions) {
            match val {
                0 => false,
                1 => true,
                _ => panic!("invalid bool discriminant"),
            }
        } else {
            val != 0
        }
    }
    pub unsafe fn cabi_dealloc(ptr: *mut u8, size: usize, align: usize) {
        if size == 0 {
            return;
        }
        let layout = alloc::Layout::from_size_align_unchecked(size, align);
        alloc::dealloc(ptr, layout);
    }
    pub use alloc_crate::alloc;
    extern crate alloc as alloc_crate;
}
/// Generates `#[no_mangle]` functions to export the specified type as the
/// root implementation of all generated traits.
///
/// For more information see the documentation of `wit_bindgen::generate!`.
///
/// ```rust
/// # macro_rules! export{ ($($t:tt)*) => (); }
/// # trait Guest {}
/// struct MyType;
///
/// impl Guest for MyType {
///     // ...
/// }
///
/// export!(MyType);
/// ```
#[allow(unused_macros)]
#[doc(hidden)]
macro_rules! __export_tools_impl {
    ($ty:ident) => {
        self::export!($ty with_types_in self);
    };
    ($ty:ident with_types_in $($path_to_types_root:tt)*) => {
        $($path_to_types_root)*:: __export_world_tools_cabi!($ty with_types_in
        $($path_to_types_root)*);
    };
}
#[doc(inline)]
pub(crate) use __export_tools_impl as export;
#[cfg(target_arch = "wasm32")]
#[link_section = "component-type:wit-bindgen:0.35.0:legion:tools:tools:encoded world"]
#[doc(hidden)]
pub static __WIT_BINDGEN_COMPONENT_TYPE: [u8; 472] = *b"\
\0asm\x0d\0\x01\0\0\x19\x16wit-component-encoding\x04\0\x07\xdc\x02\x01A\x02\x01\
A\x16\x01r\x01\x07messages\x03\0\x0btools-error\x03\0\0\x01r\x01\x0egenerate-dwa\
rf\x7f\x03\0\x0dencode-config\x03\0\x02\x01r\x03\x0dskeleton-only\x7f\x0cname-un\
named\x7f\x11fold-instructions\x7f\x03\0\x0ddecode-config\x03\0\x04\x01o\x02ss\x01\
p\x06\x01r\x02\x04names\x04shim\x07\x03\0\x0fpolyfill-config\x03\0\x08\x01p}\x01\
j\x01\x0a\x01\x01\x01@\x02\x05inputs\x06config\x03\0\x0b\x04\0\x0awat-encode\x01\
\x0c\x01j\x01s\x01\x01\x01@\x02\x05input\x0a\x06config\x05\0\x0d\x04\0\x0bwasm-d\
ecode\x01\x0e\x01o\x02s\x0a\x01p\x0f\x01j\x01\x10\x01\x01\x01@\x02\x05input\x0a\x06\
config\x09\0\x11\x04\0\x0dwasi-polyfill\x01\x12\x04\0\x12legion:tools/tools\x04\0\
\x0b\x0b\x01\0\x05tools\x03\0\0\0G\x09producers\x01\x0cprocessed-by\x02\x0dwit-c\
omponent\x070.220.0\x10wit-bindgen-rust\x060.35.0";
#[inline(never)]
#[doc(hidden)]
pub fn __link_custom_section_describing_imports() {
    wit_bindgen_rt::maybe_link_cabi_realloc();
}
