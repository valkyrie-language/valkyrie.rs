#[allow(dead_code)]
pub mod valkyrie {
    #[allow(dead_code)]
    pub mod valkyrie_legacy {
        #[allow(dead_code, clippy::all)]
        pub mod ast {
            #[used]
            #[doc(hidden)]
            static __FORCE_SECTION_REF: fn() = super::super::super::__link_custom_section_describing_imports;
            /// The kind of namespace
            #[repr(u8)]
            #[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
            pub enum NamespaceKind {
                /// Main namespace where definitions and imports can be shared
                ///
                /// `namespace foo` and file name is `_.valkyrie`
                Main,
                /// Independent namespace, isolated definitions, except public and main definitions
                ///
                /// `namespace foo`
                Standalone,
                /// This is a test file, only available in the test environment
                ///
                /// `namespace? foo`
                Test,
                /// Temporarily remove a file
                ///
                /// `namespace* foo`
                Hide,
            }
            impl ::core::fmt::Debug for NamespaceKind {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    match self {
                        NamespaceKind::Main => f.debug_tuple("NamespaceKind::Main").finish(),
                        NamespaceKind::Standalone => f.debug_tuple("NamespaceKind::Standalone").finish(),
                        NamespaceKind::Test => f.debug_tuple("NamespaceKind::Test").finish(),
                        NamespaceKind::Hide => f.debug_tuple("NamespaceKind::Hide").finish(),
                    }
                }
            }
            impl NamespaceKind {
                #[doc(hidden)]
                pub unsafe fn _lift(val: u8) -> NamespaceKind {
                    if !cfg!(debug_assertions) {
                        return ::core::mem::transmute(val);
                    }
                    match val {
                        0 => NamespaceKind::Main,
                        1 => NamespaceKind::Standalone,
                        2 => NamespaceKind::Test,
                        3 => NamespaceKind::Hide,
                        _ => panic!("invalid enum discriminant"),
                    }
                }
            }
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
        }
    }
}
#[allow(dead_code)]
pub mod exports {
    #[allow(dead_code)]
    pub mod valkyrie {
        #[allow(dead_code)]
        pub mod valkyrie_legacy {
            #[allow(dead_code, clippy::all)]
            pub mod string_pool {
                #[used]
                #[doc(hidden)]
                static __FORCE_SECTION_REF: fn() = super::super::super::super::__link_custom_section_describing_imports;
                use super::super::super::super::_rt;
                #[derive(Debug)]
                #[repr(transparent)]
                pub struct StringPool {
                    handle: _rt::Resource<StringPool>,
                }
                type _StringPoolRep<T> = Option<T>;
                impl StringPool {
                    /// Creates a new resource from the specified representation.
                    ///
                    /// This function will create a new resource handle by moving `val` onto
                    /// the heap and then passing that heap pointer to the component model to
                    /// create a handle. The owned handle is then returned as `StringPool`.
                    pub fn new<T: GuestStringPool>(val: T) -> Self {
                        Self::type_guard::<T>();
                        let val: _StringPoolRep<T> = Some(val);
                        let ptr: *mut _StringPoolRep<T> = _rt::Box::into_raw(_rt::Box::new(val));
                        unsafe { Self::from_handle(T::_resource_new(ptr.cast())) }
                    }
                    /// Gets access to the underlying `T` which represents this resource.
                    pub fn get<T: GuestStringPool>(&self) -> &T {
                        let ptr = unsafe { &*self.as_ptr::<T>() };
                        ptr.as_ref().unwrap()
                    }
                    /// Gets mutable access to the underlying `T` which represents this
                    /// resource.
                    pub fn get_mut<T: GuestStringPool>(&mut self) -> &mut T {
                        let ptr = unsafe { &mut *self.as_ptr::<T>() };
                        ptr.as_mut().unwrap()
                    }
                    /// Consumes this resource and returns the underlying `T`.
                    pub fn into_inner<T: GuestStringPool>(self) -> T {
                        let ptr = unsafe { &mut *self.as_ptr::<T>() };
                        ptr.take().unwrap()
                    }
                    #[doc(hidden)]
                    pub unsafe fn from_handle(handle: u32) -> Self {
                        Self { handle: _rt::Resource::from_handle(handle) }
                    }
                    #[doc(hidden)]
                    pub fn take_handle(&self) -> u32 {
                        _rt::Resource::take_handle(&self.handle)
                    }
                    #[doc(hidden)]
                    pub fn handle(&self) -> u32 {
                        _rt::Resource::handle(&self.handle)
                    }
                    #[doc(hidden)]
                    fn type_guard<T: 'static>() {
                        use core::any::TypeId;
                        static mut LAST_TYPE: Option<TypeId> = None;
                        unsafe {
                            assert!(!cfg!(target_feature = "atomics"));
                            let id = TypeId::of::<T>();
                            match LAST_TYPE {
                                Some(ty) => {
                                    assert!(ty == id, "cannot use two types with this resource type")
                                }
                                None => LAST_TYPE = Some(id),
                            }
                        }
                    }
                    #[doc(hidden)]
                    pub unsafe fn dtor<T: 'static>(handle: *mut u8) {
                        Self::type_guard::<T>();
                        let _ = _rt::Box::from_raw(handle as *mut _StringPoolRep<T>);
                    }
                    fn as_ptr<T: GuestStringPool>(&self) -> *mut _StringPoolRep<T> {
                        StringPool::type_guard::<T>();
                        T::_resource_rep(self.handle()).cast()
                    }
                }
                /// A borrowed version of [`StringPool`] which represents a borrowed value
                /// with the lifetime `'a`.
                #[derive(Debug)]
                #[repr(transparent)]
                pub struct StringPoolBorrow<'a> {
                    rep: *mut u8,
                    _marker: core::marker::PhantomData<&'a StringPool>,
                }
                impl<'a> StringPoolBorrow<'a> {
                    #[doc(hidden)]
                    pub unsafe fn lift(rep: usize) -> Self {
                        Self { rep: rep as *mut u8, _marker: core::marker::PhantomData }
                    }
                    /// Gets access to the underlying `T` in this resource.
                    pub fn get<T: GuestStringPool>(&self) -> &T {
                        let ptr = unsafe { &mut *self.as_ptr::<T>() };
                        ptr.as_ref().unwrap()
                    }
                    fn as_ptr<T: 'static>(&self) -> *mut _StringPoolRep<T> {
                        StringPool::type_guard::<T>();
                        self.rep.cast()
                    }
                }
                unsafe impl _rt::WasmResource for StringPool {
                    #[inline]
                    unsafe fn drop(_handle: u32) {
                        #[cfg(not(target_arch = "wasm32"))]
                        unreachable!();
                        #[cfg(target_arch = "wasm32")]
                        {
                            #[link(wasm_import_module = "[export]valkyrie:valkyrie-legacy/string-pool")]
                            extern "C" {
                                #[link_name = "[resource-drop]string-pool"]
                                fn drop(_: u32);
                            }
                            drop(_handle);
                        }
                    }
                }
                #[derive(Debug)]
                #[repr(transparent)]
                pub struct Identifier {
                    handle: _rt::Resource<Identifier>,
                }
                type _IdentifierRep<T> = Option<T>;
                impl Identifier {
                    /// Creates a new resource from the specified representation.
                    ///
                    /// This function will create a new resource handle by moving `val` onto
                    /// the heap and then passing that heap pointer to the component model to
                    /// create a handle. The owned handle is then returned as `Identifier`.
                    pub fn new<T: GuestIdentifier>(val: T) -> Self {
                        Self::type_guard::<T>();
                        let val: _IdentifierRep<T> = Some(val);
                        let ptr: *mut _IdentifierRep<T> = _rt::Box::into_raw(_rt::Box::new(val));
                        unsafe { Self::from_handle(T::_resource_new(ptr.cast())) }
                    }
                    /// Gets access to the underlying `T` which represents this resource.
                    pub fn get<T: GuestIdentifier>(&self) -> &T {
                        let ptr = unsafe { &*self.as_ptr::<T>() };
                        ptr.as_ref().unwrap()
                    }
                    /// Gets mutable access to the underlying `T` which represents this
                    /// resource.
                    pub fn get_mut<T: GuestIdentifier>(&mut self) -> &mut T {
                        let ptr = unsafe { &mut *self.as_ptr::<T>() };
                        ptr.as_mut().unwrap()
                    }
                    /// Consumes this resource and returns the underlying `T`.
                    pub fn into_inner<T: GuestIdentifier>(self) -> T {
                        let ptr = unsafe { &mut *self.as_ptr::<T>() };
                        ptr.take().unwrap()
                    }
                    #[doc(hidden)]
                    pub unsafe fn from_handle(handle: u32) -> Self {
                        Self { handle: _rt::Resource::from_handle(handle) }
                    }
                    #[doc(hidden)]
                    pub fn take_handle(&self) -> u32 {
                        _rt::Resource::take_handle(&self.handle)
                    }
                    #[doc(hidden)]
                    pub fn handle(&self) -> u32 {
                        _rt::Resource::handle(&self.handle)
                    }
                    #[doc(hidden)]
                    fn type_guard<T: 'static>() {
                        use core::any::TypeId;
                        static mut LAST_TYPE: Option<TypeId> = None;
                        unsafe {
                            assert!(!cfg!(target_feature = "atomics"));
                            let id = TypeId::of::<T>();
                            match LAST_TYPE {
                                Some(ty) => {
                                    assert!(ty == id, "cannot use two types with this resource type")
                                }
                                None => LAST_TYPE = Some(id),
                            }
                        }
                    }
                    #[doc(hidden)]
                    pub unsafe fn dtor<T: 'static>(handle: *mut u8) {
                        Self::type_guard::<T>();
                        let _ = _rt::Box::from_raw(handle as *mut _IdentifierRep<T>);
                    }
                    fn as_ptr<T: GuestIdentifier>(&self) -> *mut _IdentifierRep<T> {
                        Identifier::type_guard::<T>();
                        T::_resource_rep(self.handle()).cast()
                    }
                }
                /// A borrowed version of [`Identifier`] which represents a borrowed value
                /// with the lifetime `'a`.
                #[derive(Debug)]
                #[repr(transparent)]
                pub struct IdentifierBorrow<'a> {
                    rep: *mut u8,
                    _marker: core::marker::PhantomData<&'a Identifier>,
                }
                impl<'a> IdentifierBorrow<'a> {
                    #[doc(hidden)]
                    pub unsafe fn lift(rep: usize) -> Self {
                        Self { rep: rep as *mut u8, _marker: core::marker::PhantomData }
                    }
                    /// Gets access to the underlying `T` in this resource.
                    pub fn get<T: GuestIdentifier>(&self) -> &T {
                        let ptr = unsafe { &mut *self.as_ptr::<T>() };
                        ptr.as_ref().unwrap()
                    }
                    fn as_ptr<T: 'static>(&self) -> *mut _IdentifierRep<T> {
                        Identifier::type_guard::<T>();
                        self.rep.cast()
                    }
                }
                unsafe impl _rt::WasmResource for Identifier {
                    #[inline]
                    unsafe fn drop(_handle: u32) {
                        #[cfg(not(target_arch = "wasm32"))]
                        unreachable!();
                        #[cfg(target_arch = "wasm32")]
                        {
                            #[link(wasm_import_module = "[export]valkyrie:valkyrie-legacy/string-pool")]
                            extern "C" {
                                #[link_name = "[resource-drop]identifier"]
                                fn drop(_: u32);
                            }
                            drop(_handle);
                        }
                    }
                }
                pub trait Guest {
                    type StringPool: GuestStringPool;
                    type Identifier: GuestIdentifier;
                }
                pub trait GuestStringPool: 'static {
                    #[doc(hidden)]
                    unsafe fn _resource_new(val: *mut u8) -> u32
                    where
                        Self: Sized,
                    {
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            let _ = val;
                            unreachable!();
                        }
                        #[cfg(target_arch = "wasm32")]
                        {
                            #[link(wasm_import_module = "[export]valkyrie:valkyrie-legacy/string-pool")]
                            extern "C" {
                                #[link_name = "[resource-new]string-pool"]
                                fn new(_: *mut u8) -> u32;
                            }
                            new(val)
                        }
                    }
                    #[doc(hidden)]
                    fn _resource_rep(handle: u32) -> *mut u8
                    where
                        Self: Sized,
                    {
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            let _ = handle;
                            unreachable!();
                        }
                        #[cfg(target_arch = "wasm32")]
                        {
                            #[link(wasm_import_module = "[export]valkyrie:valkyrie-legacy/string-pool")]
                            extern "C" {
                                #[link_name = "[resource-rep]string-pool"]
                                fn rep(_: u32) -> *mut u8;
                            }
                            unsafe { rep(handle) }
                        }
                    }
                }
                pub trait GuestIdentifier: 'static {
                    #[doc(hidden)]
                    unsafe fn _resource_new(val: *mut u8) -> u32
                    where
                        Self: Sized,
                    {
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            let _ = val;
                            unreachable!();
                        }
                        #[cfg(target_arch = "wasm32")]
                        {
                            #[link(wasm_import_module = "[export]valkyrie:valkyrie-legacy/string-pool")]
                            extern "C" {
                                #[link_name = "[resource-new]identifier"]
                                fn new(_: *mut u8) -> u32;
                            }
                            new(val)
                        }
                    }
                    #[doc(hidden)]
                    fn _resource_rep(handle: u32) -> *mut u8
                    where
                        Self: Sized,
                    {
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            let _ = handle;
                            unreachable!();
                        }
                        #[cfg(target_arch = "wasm32")]
                        {
                            #[link(wasm_import_module = "[export]valkyrie:valkyrie-legacy/string-pool")]
                            extern "C" {
                                #[link_name = "[resource-rep]identifier"]
                                fn rep(_: u32) -> *mut u8;
                            }
                            unsafe { rep(handle) }
                        }
                    }
                }
                #[doc(hidden)]
                macro_rules! __export_valkyrie_valkyrie_legacy_string_pool_cabi {
                    ($ty:ident with_types_in $($path_to_types:tt)*) => {
                        const _ : () = { const _ : () = { #[doc(hidden)] #[export_name =
                        "valkyrie:valkyrie-legacy/string-pool#[dtor]string-pool"]
                        #[allow(non_snake_case)] unsafe extern "C" fn dtor(rep : * mut
                        u8) { $($path_to_types)*:: StringPool::dtor::< <$ty as
                        $($path_to_types)*:: Guest >::StringPool > (rep) } }; const _ :
                        () = { #[doc(hidden)] #[export_name =
                        "valkyrie:valkyrie-legacy/string-pool#[dtor]identifier"]
                        #[allow(non_snake_case)] unsafe extern "C" fn dtor(rep : * mut
                        u8) { $($path_to_types)*:: Identifier::dtor::< <$ty as
                        $($path_to_types)*:: Guest >::Identifier > (rep) } }; };
                    };
                }
                #[doc(hidden)]
                pub(crate) use __export_valkyrie_valkyrie_legacy_string_pool_cabi;
            }
            #[allow(dead_code, clippy::all)]
            pub mod source_pool {
                #[used]
                #[doc(hidden)]
                static __FORCE_SECTION_REF: fn() = super::super::super::super::__link_custom_section_describing_imports;
                use super::super::super::super::_rt;
                #[derive(Debug)]
                #[repr(transparent)]
                pub struct SourcePool {
                    handle: _rt::Resource<SourcePool>,
                }
                type _SourcePoolRep<T> = Option<T>;
                impl SourcePool {
                    /// Creates a new resource from the specified representation.
                    ///
                    /// This function will create a new resource handle by moving `val` onto
                    /// the heap and then passing that heap pointer to the component model to
                    /// create a handle. The owned handle is then returned as `SourcePool`.
                    pub fn new<T: GuestSourcePool>(val: T) -> Self {
                        Self::type_guard::<T>();
                        let val: _SourcePoolRep<T> = Some(val);
                        let ptr: *mut _SourcePoolRep<T> = _rt::Box::into_raw(_rt::Box::new(val));
                        unsafe { Self::from_handle(T::_resource_new(ptr.cast())) }
                    }
                    /// Gets access to the underlying `T` which represents this resource.
                    pub fn get<T: GuestSourcePool>(&self) -> &T {
                        let ptr = unsafe { &*self.as_ptr::<T>() };
                        ptr.as_ref().unwrap()
                    }
                    /// Gets mutable access to the underlying `T` which represents this
                    /// resource.
                    pub fn get_mut<T: GuestSourcePool>(&mut self) -> &mut T {
                        let ptr = unsafe { &mut *self.as_ptr::<T>() };
                        ptr.as_mut().unwrap()
                    }
                    /// Consumes this resource and returns the underlying `T`.
                    pub fn into_inner<T: GuestSourcePool>(self) -> T {
                        let ptr = unsafe { &mut *self.as_ptr::<T>() };
                        ptr.take().unwrap()
                    }
                    #[doc(hidden)]
                    pub unsafe fn from_handle(handle: u32) -> Self {
                        Self { handle: _rt::Resource::from_handle(handle) }
                    }
                    #[doc(hidden)]
                    pub fn take_handle(&self) -> u32 {
                        _rt::Resource::take_handle(&self.handle)
                    }
                    #[doc(hidden)]
                    pub fn handle(&self) -> u32 {
                        _rt::Resource::handle(&self.handle)
                    }
                    #[doc(hidden)]
                    fn type_guard<T: 'static>() {
                        use core::any::TypeId;
                        static mut LAST_TYPE: Option<TypeId> = None;
                        unsafe {
                            assert!(!cfg!(target_feature = "atomics"));
                            let id = TypeId::of::<T>();
                            match LAST_TYPE {
                                Some(ty) => {
                                    assert!(ty == id, "cannot use two types with this resource type")
                                }
                                None => LAST_TYPE = Some(id),
                            }
                        }
                    }
                    #[doc(hidden)]
                    pub unsafe fn dtor<T: 'static>(handle: *mut u8) {
                        Self::type_guard::<T>();
                        let _ = _rt::Box::from_raw(handle as *mut _SourcePoolRep<T>);
                    }
                    fn as_ptr<T: GuestSourcePool>(&self) -> *mut _SourcePoolRep<T> {
                        SourcePool::type_guard::<T>();
                        T::_resource_rep(self.handle()).cast()
                    }
                }
                /// A borrowed version of [`SourcePool`] which represents a borrowed value
                /// with the lifetime `'a`.
                #[derive(Debug)]
                #[repr(transparent)]
                pub struct SourcePoolBorrow<'a> {
                    rep: *mut u8,
                    _marker: core::marker::PhantomData<&'a SourcePool>,
                }
                impl<'a> SourcePoolBorrow<'a> {
                    #[doc(hidden)]
                    pub unsafe fn lift(rep: usize) -> Self {
                        Self { rep: rep as *mut u8, _marker: core::marker::PhantomData }
                    }
                    /// Gets access to the underlying `T` in this resource.
                    pub fn get<T: GuestSourcePool>(&self) -> &T {
                        let ptr = unsafe { &mut *self.as_ptr::<T>() };
                        ptr.as_ref().unwrap()
                    }
                    fn as_ptr<T: 'static>(&self) -> *mut _SourcePoolRep<T> {
                        SourcePool::type_guard::<T>();
                        self.rep.cast()
                    }
                }
                unsafe impl _rt::WasmResource for SourcePool {
                    #[inline]
                    unsafe fn drop(_handle: u32) {
                        #[cfg(not(target_arch = "wasm32"))]
                        unreachable!();
                        #[cfg(target_arch = "wasm32")]
                        {
                            #[link(wasm_import_module = "[export]valkyrie:valkyrie-legacy/source-pool")]
                            extern "C" {
                                #[link_name = "[resource-drop]source-pool"]
                                fn drop(_: u32);
                            }
                            drop(_handle);
                        }
                    }
                }
                #[derive(Debug)]
                #[repr(transparent)]
                pub struct SourceId {
                    handle: _rt::Resource<SourceId>,
                }
                type _SourceIdRep<T> = Option<T>;
                impl SourceId {
                    /// Creates a new resource from the specified representation.
                    ///
                    /// This function will create a new resource handle by moving `val` onto
                    /// the heap and then passing that heap pointer to the component model to
                    /// create a handle. The owned handle is then returned as `SourceId`.
                    pub fn new<T: GuestSourceId>(val: T) -> Self {
                        Self::type_guard::<T>();
                        let val: _SourceIdRep<T> = Some(val);
                        let ptr: *mut _SourceIdRep<T> = _rt::Box::into_raw(_rt::Box::new(val));
                        unsafe { Self::from_handle(T::_resource_new(ptr.cast())) }
                    }
                    /// Gets access to the underlying `T` which represents this resource.
                    pub fn get<T: GuestSourceId>(&self) -> &T {
                        let ptr = unsafe { &*self.as_ptr::<T>() };
                        ptr.as_ref().unwrap()
                    }
                    /// Gets mutable access to the underlying `T` which represents this
                    /// resource.
                    pub fn get_mut<T: GuestSourceId>(&mut self) -> &mut T {
                        let ptr = unsafe { &mut *self.as_ptr::<T>() };
                        ptr.as_mut().unwrap()
                    }
                    /// Consumes this resource and returns the underlying `T`.
                    pub fn into_inner<T: GuestSourceId>(self) -> T {
                        let ptr = unsafe { &mut *self.as_ptr::<T>() };
                        ptr.take().unwrap()
                    }
                    #[doc(hidden)]
                    pub unsafe fn from_handle(handle: u32) -> Self {
                        Self { handle: _rt::Resource::from_handle(handle) }
                    }
                    #[doc(hidden)]
                    pub fn take_handle(&self) -> u32 {
                        _rt::Resource::take_handle(&self.handle)
                    }
                    #[doc(hidden)]
                    pub fn handle(&self) -> u32 {
                        _rt::Resource::handle(&self.handle)
                    }
                    #[doc(hidden)]
                    fn type_guard<T: 'static>() {
                        use core::any::TypeId;
                        static mut LAST_TYPE: Option<TypeId> = None;
                        unsafe {
                            assert!(!cfg!(target_feature = "atomics"));
                            let id = TypeId::of::<T>();
                            match LAST_TYPE {
                                Some(ty) => {
                                    assert!(ty == id, "cannot use two types with this resource type")
                                }
                                None => LAST_TYPE = Some(id),
                            }
                        }
                    }
                    #[doc(hidden)]
                    pub unsafe fn dtor<T: 'static>(handle: *mut u8) {
                        Self::type_guard::<T>();
                        let _ = _rt::Box::from_raw(handle as *mut _SourceIdRep<T>);
                    }
                    fn as_ptr<T: GuestSourceId>(&self) -> *mut _SourceIdRep<T> {
                        SourceId::type_guard::<T>();
                        T::_resource_rep(self.handle()).cast()
                    }
                }
                /// A borrowed version of [`SourceId`] which represents a borrowed value
                /// with the lifetime `'a`.
                #[derive(Debug)]
                #[repr(transparent)]
                pub struct SourceIdBorrow<'a> {
                    rep: *mut u8,
                    _marker: core::marker::PhantomData<&'a SourceId>,
                }
                impl<'a> SourceIdBorrow<'a> {
                    #[doc(hidden)]
                    pub unsafe fn lift(rep: usize) -> Self {
                        Self { rep: rep as *mut u8, _marker: core::marker::PhantomData }
                    }
                    /// Gets access to the underlying `T` in this resource.
                    pub fn get<T: GuestSourceId>(&self) -> &T {
                        let ptr = unsafe { &mut *self.as_ptr::<T>() };
                        ptr.as_ref().unwrap()
                    }
                    fn as_ptr<T: 'static>(&self) -> *mut _SourceIdRep<T> {
                        SourceId::type_guard::<T>();
                        self.rep.cast()
                    }
                }
                unsafe impl _rt::WasmResource for SourceId {
                    #[inline]
                    unsafe fn drop(_handle: u32) {
                        #[cfg(not(target_arch = "wasm32"))]
                        unreachable!();
                        #[cfg(target_arch = "wasm32")]
                        {
                            #[link(wasm_import_module = "[export]valkyrie:valkyrie-legacy/source-pool")]
                            extern "C" {
                                #[link_name = "[resource-drop]source-id"]
                                fn drop(_: u32);
                            }
                            drop(_handle);
                        }
                    }
                }
                pub trait Guest {
                    type SourcePool: GuestSourcePool;
                    type SourceId: GuestSourceId;
                }
                pub trait GuestSourcePool: 'static {
                    #[doc(hidden)]
                    unsafe fn _resource_new(val: *mut u8) -> u32
                    where
                        Self: Sized,
                    {
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            let _ = val;
                            unreachable!();
                        }
                        #[cfg(target_arch = "wasm32")]
                        {
                            #[link(wasm_import_module = "[export]valkyrie:valkyrie-legacy/source-pool")]
                            extern "C" {
                                #[link_name = "[resource-new]source-pool"]
                                fn new(_: *mut u8) -> u32;
                            }
                            new(val)
                        }
                    }
                    #[doc(hidden)]
                    fn _resource_rep(handle: u32) -> *mut u8
                    where
                        Self: Sized,
                    {
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            let _ = handle;
                            unreachable!();
                        }
                        #[cfg(target_arch = "wasm32")]
                        {
                            #[link(wasm_import_module = "[export]valkyrie:valkyrie-legacy/source-pool")]
                            extern "C" {
                                #[link_name = "[resource-rep]source-pool"]
                                fn rep(_: u32) -> *mut u8;
                            }
                            unsafe { rep(handle) }
                        }
                    }
                }
                pub trait GuestSourceId: 'static {
                    #[doc(hidden)]
                    unsafe fn _resource_new(val: *mut u8) -> u32
                    where
                        Self: Sized,
                    {
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            let _ = val;
                            unreachable!();
                        }
                        #[cfg(target_arch = "wasm32")]
                        {
                            #[link(wasm_import_module = "[export]valkyrie:valkyrie-legacy/source-pool")]
                            extern "C" {
                                #[link_name = "[resource-new]source-id"]
                                fn new(_: *mut u8) -> u32;
                            }
                            new(val)
                        }
                    }
                    #[doc(hidden)]
                    fn _resource_rep(handle: u32) -> *mut u8
                    where
                        Self: Sized,
                    {
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            let _ = handle;
                            unreachable!();
                        }
                        #[cfg(target_arch = "wasm32")]
                        {
                            #[link(wasm_import_module = "[export]valkyrie:valkyrie-legacy/source-pool")]
                            extern "C" {
                                #[link_name = "[resource-rep]source-id"]
                                fn rep(_: u32) -> *mut u8;
                            }
                            unsafe { rep(handle) }
                        }
                    }
                }
                #[doc(hidden)]
                macro_rules! __export_valkyrie_valkyrie_legacy_source_pool_cabi {
                    ($ty:ident with_types_in $($path_to_types:tt)*) => {
                        const _ : () = { const _ : () = { #[doc(hidden)] #[export_name =
                        "valkyrie:valkyrie-legacy/source-pool#[dtor]source-pool"]
                        #[allow(non_snake_case)] unsafe extern "C" fn dtor(rep : * mut
                        u8) { $($path_to_types)*:: SourcePool::dtor::< <$ty as
                        $($path_to_types)*:: Guest >::SourcePool > (rep) } }; const _ :
                        () = { #[doc(hidden)] #[export_name =
                        "valkyrie:valkyrie-legacy/source-pool#[dtor]source-id"]
                        #[allow(non_snake_case)] unsafe extern "C" fn dtor(rep : * mut
                        u8) { $($path_to_types)*:: SourceId::dtor::< <$ty as
                        $($path_to_types)*:: Guest >::SourceId > (rep) } }; };
                    };
                }
                #[doc(hidden)]
                pub(crate) use __export_valkyrie_valkyrie_legacy_source_pool_cabi;
            }
        }
    }
}
mod _rt {
    use core::{
        fmt, marker,
        sync::atomic::{AtomicU32, Ordering::Relaxed},
    };
    /// A type which represents a component model resource, either imported or
    /// exported into this component.
    ///
    /// This is a low-level wrapper which handles the lifetime of the resource
    /// (namely this has a destructor). The `T` provided defines the component model
    /// intrinsics that this wrapper uses.
    ///
    /// One of the chief purposes of this type is to provide `Deref` implementations
    /// to access the underlying data when it is owned.
    ///
    /// This type is primarily used in generated code for exported and imported
    /// resources.
    #[repr(transparent)]
    pub struct Resource<T: WasmResource> {
        handle: AtomicU32,
        _marker: marker::PhantomData<T>,
    }
    /// A trait which all wasm resources implement, namely providing the ability to
    /// drop a resource.
    ///
    /// This generally is implemented by generated code, not user-facing code.
    #[allow(clippy::missing_safety_doc)]
    pub unsafe trait WasmResource {
        /// Invokes the `[resource-drop]...` intrinsic.
        unsafe fn drop(handle: u32);
    }
    impl<T: WasmResource> Resource<T> {
        #[doc(hidden)]
        pub unsafe fn from_handle(handle: u32) -> Self {
            debug_assert!(handle != u32::MAX);
            Self { handle: AtomicU32::new(handle), _marker: marker::PhantomData }
        }
        /// Takes ownership of the handle owned by `resource`.
        ///
        /// Note that this ideally would be `into_handle` taking `Resource<T>` by
        /// ownership. The code generator does not enable that in all situations,
        /// unfortunately, so this is provided instead.
        ///
        /// Also note that `take_handle` is in theory only ever called on values
        /// owned by a generated function. For example a generated function might
        /// take `Resource<T>` as an argument but then call `take_handle` on a
        /// reference to that argument. In that sense the dynamic nature of
        /// `take_handle` should only be exposed internally to generated code, not
        /// to user code.
        #[doc(hidden)]
        pub fn take_handle(resource: &Resource<T>) -> u32 {
            resource.handle.swap(u32::MAX, Relaxed)
        }
        #[doc(hidden)]
        pub fn handle(resource: &Resource<T>) -> u32 {
            resource.handle.load(Relaxed)
        }
    }
    impl<T: WasmResource> fmt::Debug for Resource<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Resource").field("handle", &self.handle).finish()
        }
    }
    impl<T: WasmResource> Drop for Resource<T> {
        fn drop(&mut self) {
            unsafe {
                match self.handle.load(Relaxed) {
                    u32::MAX => {}
                    other => T::drop(other),
                }
            }
        }
    }
    pub use alloc_crate::boxed::Box;
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
macro_rules! __export_types_impl {
    ($ty:ident) => {
        self::export!($ty with_types_in self);
    };
    ($ty:ident with_types_in $($path_to_types_root:tt)*) => {
        $($path_to_types_root)*::
        exports::valkyrie::valkyrie_legacy::string_pool::__export_valkyrie_valkyrie_legacy_string_pool_cabi!($ty
        with_types_in $($path_to_types_root)*::
        exports::valkyrie::valkyrie_legacy::string_pool); $($path_to_types_root)*::
        exports::valkyrie::valkyrie_legacy::source_pool::__export_valkyrie_valkyrie_legacy_source_pool_cabi!($ty
        with_types_in $($path_to_types_root)*::
        exports::valkyrie::valkyrie_legacy::source_pool);
    };
}
#[doc(inline)]
pub(crate) use __export_types_impl as export;
#[cfg(target_arch = "wasm32")]
#[link_section = "component-type:wit-bindgen:0.35.0:valkyrie:valkyrie-legacy:types:encoded world"]
#[doc(hidden)]
pub static __WIT_BINDGEN_COMPONENT_TYPE: [u8; 530] = *b"\
\0asm\x0d\0\x01\0\0\x19\x16wit-component-encoding\x04\0\x07\x96\x03\x01A\x02\x01\
A\x06\x01B\x06\x01m\x04\x04main\x0astandalone\x04test\x04hide\x04\0\x0enamespace\
-kind\x03\0\0\x01m\x03\x04auto\x0casynchronous\x0bsynchronous\x04\0\x11asynchron\
ous-kind\x03\0\x02\x01m\x02\x05macro\x05micro\x04\0\x0dfunction-kind\x03\0\x04\x03\
\0\x1cvalkyrie:valkyrie-legacy/ast\x05\0\x01B\x02\x04\0\x0bstring-pool\x03\x01\x04\
\0\x0aidentifier\x03\x01\x04\0$valkyrie:valkyrie-legacy/string-pool\x05\x01\x01B\
\x05\x04\0\x0bsource-pool\x03\x01\x04\0\x09source-id\x03\x01\x01i\x01\x01r\x03\x04\
file\x02\x05starty\x03endy\x04\0\x0bsource-span\x03\0\x03\x04\0$valkyrie:valkyri\
e-legacy/source-pool\x05\x02\x04\0\x1evalkyrie:valkyrie-legacy/types\x04\0\x0b\x0b\
\x01\0\x05types\x03\0\0\0G\x09producers\x01\x0cprocessed-by\x02\x0dwit-component\
\x070.220.0\x10wit-bindgen-rust\x060.35.0";
#[inline(never)]
#[doc(hidden)]
pub fn __link_custom_section_describing_imports() {
    wit_bindgen_rt::maybe_link_cabi_realloc();
}
