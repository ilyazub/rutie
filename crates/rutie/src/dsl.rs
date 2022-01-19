/// Creates Rust structure for new Ruby class
///
/// This macro does not define an actual Ruby class. It only creates structs for using
/// the class in Rust. To define the class in Ruby, use `Class` structure.
///
/// # Examples
///
/// ```
/// #[macro_use]
/// extern crate rutie;
///
/// use rutie::{Class, RString, Object, VM};
///
/// class!(Greeter);
///
/// methods!(
///     Greeter,
///     rtself,
///
///     fn anonymous_greeting() -> RString {
///         RString::new_utf8("Hello stranger!")
///     }
///
///     fn friendly_greeting(name: RString) -> RString {
///         let name = name
///             .map(|name| name.to_string())
///             .unwrap_or("Anonymous".to_string());
///
///         let greeting = format!("Hello dear {}!", name);
///
///         RString::new_utf8(&greeting)
///     }
/// );
///
/// fn main() {
///     # VM::init();
///     Class::new("Greeter", None).define(|klass| {
///         klass.def("anonymous_greeting", anonymous_greeting);
///         klass.def("friendly_greeting", friendly_greeting);
///     });
/// }
/// ```
///
/// Ruby:
///
/// ```ruby
/// class Greeter
///   def anonymous_greeting
///     'Hello stranger!'
///   end
///
///   def friendly_greeting(name)
///     default_name = 'Anonymous'
///
///     name = defaut_name unless name.is_a?(String)
///
///     "Hello dear #{name}"
///   end
/// end
/// ```
#[macro_export]
macro_rules! class {
    ($class: ident) => {
        #[repr(C)]
        #[derive(Debug, PartialEq)]
        pub struct $class {
            value: $crate::types::Value,
        }

        impl From<$crate::types::Value> for $class {
            fn from(value: $crate::types::Value) -> Self {
                $class { value: value }
            }
        }

        impl $crate::Object for $class {
            #[inline]
            fn value(&self) -> $crate::types::Value {
                self.value
            }
        }
    }
}

/// Creates Rust structure for new Ruby module
///
/// This macro does not define an actual Ruby module. It only creates structs for using
/// the module in Rust. To define the module in Ruby, use `Module` structure.
///
/// # Examples
///
/// ```
/// #[macro_use]
/// extern crate rutie;
///
/// use rutie::{Module, RString, Object, VM};
///
/// module!(Greeter);
///
/// methods!(
///     Greeter,
///     rtself,
///
///     fn anonymous_greeting() -> RString {
///         RString::new_utf8("Hello stranger!")
///     }
///
///     fn friendly_greeting(name: RString) -> RString {
///         let name = name
///             .map(|name| name.to_string())
///             .unwrap_or("Anonymous".to_string());
///
///         let greeting = format!("Hello dear {}!", name);
///
///         RString::new_utf8(&greeting)
///     }
/// );
///
/// fn main() {
///     # VM::init();
///     Module::new("Greeter").define(|klass| {
///         klass.def("anonymous_greeting", anonymous_greeting);
///         klass.def("friendly_greeting", friendly_greeting);
///     });
/// }
/// ```
///
/// Ruby:
///
/// ```ruby
/// module Greeter
///   def anonymous_greeting
///     'Hello stranger!'
///   end
///
///   def friendly_greeting(name)
///     default_name = 'Anonymous'
///
///     name = defaut_name unless name.is_a?(String)
///
///     "Hello dear #{name}"
///   end
/// end
/// ```
#[macro_export]
macro_rules! module {
    ($module: ident) => {
        #[repr(C)]
        #[derive(Debug, PartialEq)]
        pub struct $module {
            value: $crate::types::Value,
        }

        impl From<$crate::types::Value> for $module {
            fn from(value: $crate::types::Value) -> Self {
                $module { value: value }
            }
        }

        impl $crate::Object for $module {
            #[inline]
            fn value(&self) -> $crate::types::Value {
                self.value
            }
        }
    }
}

/// Creates unsafe callbacks for Ruby methods
///
/// This macro is unsafe, because:
///
///  - it uses automatic unsafe conversions for arguments
///     (no guarantee that Ruby objects match the types which you expect);
///  - no bound checks for the array of provided arguments
///     (no guarantee that all the expected arguments are provided);
///
/// That is why creating callbacks in unsafe way may cause panics.
///
/// Due to the same reasons unsafe callbacks are faster.
///
/// Use it when:
///
///  - you own the Ruby code which passes arguments to callback;
///  - you are sure that all the object has correct type;
///  - you are sure that all the required arguments are provided;
///  - Ruby code has a good test coverage.
///
/// # Examples
///
/// ```
/// #[macro_use]
/// extern crate rutie;
///
/// use rutie::{Boolean, Class, Fixnum, Object, RString, VM};
///
/// // Creates `string_length_equals` functions
/// unsafe_methods!(
///     RString, // type of `self` object
///     rtself, // name of `self` object which will be used in methods
///
///     fn string_length_equals(expected_length: Fixnum) -> Boolean {
///         let real_length = rtself.to_str().len() as i64;
///
///         Boolean::new(expected_length.to_i64() == real_length)
///     }
/// );
///
/// fn main() {
///     # VM::init();
///     Class::from_existing("String").define(|klass| {
///         klass.def("length_equals?", string_length_equals);
///     });
/// }
/// ```
///
/// Ruby:
///
/// ```ruby
/// class String
///   def blank?
///     # ...
///   end
///
///   def length_equals?(expected_length)
///     # ...
///   end
/// end
/// ```
#[macro_export]
macro_rules! unsafe_methods {
    (
        $rtself_class: ty,
        $rtself_name: ident,
        $(
            fn $method_name: ident
            ($($arg_name: ident: $arg_type: ty),*) -> $return_type: ty $body: block
            $(,)?
        )*
    ) => {
        $(
            #[allow(unused_mut)]
            pub extern fn $method_name(argc: $crate::types::Argc,
                                       argv: *const $crate::AnyObject,
                                       mut $rtself_name: $rtself_class) -> $return_type {
                let _arguments = $crate::util::parse_arguments(argc, argv);
                let mut _i = 0;

                $(
                    let $arg_name = unsafe {
                        <$crate::AnyObject as $crate::Object>
                            ::to::<$arg_type>(&_arguments[_i])
                    };

                    _i += 1;
                )*

                $body
            }
        )*
    }
}

/// Creates callbacks for Ruby methods
///
/// Unlike `unsafe_methods!`, this macro is safe, because:
///
///  - it uses safe conversions of arguments (`Object::try_convert_to()`);
///  - it checks if arguments are present;
///
/// Each argument will have type `Result<Object, AnyException>`.
///
/// For example, if you declare `number: Fixnum` in the method definition, it will have actual
/// type `number: Result<Fixnum, AnyException>`.
///
/// See examples below and docs for `Object::try_convert_to()` for more information.
///
/// # Examples
///
/// To launch a server in Rust, you plan to write a simple `Server` class
///
/// ```ruby
/// class Server
///   def start(address)
///     # ...
///   end
/// end
/// ```
///
/// The `address` must be `Hash` with the following structure:
///
/// ```ruby
/// {
///   host: 'localhost',
///   port: 8080,
/// }
/// ```
///
/// You want to extract port from it. Default port is `8080` in case when:
///
///  - `address` is not a `Hash`
///  - `address[:port]` is not present
///  - `address[:port]` is not a `Fixnum`
///
/// ```
/// #[macro_use]
/// extern crate rutie;
///
/// use rutie::{Class, Fixnum, Hash, NilClass, Object, Symbol, VM};
///
/// class!(Server);
///
/// methods!(
///     Server,
///     rtself,
///
///     fn start(address: Hash) -> NilClass {
///         let default_port = 8080;
///
///         let port = address
///             .map(|hash| hash.at(&Symbol::new("port")))
///             .and_then(|port| port.try_convert_to::<Fixnum>())
///             .map(|port| port.to_i64())
///             .unwrap_or(default_port);
///
///         // Start server...
///
///         NilClass::new()
///     }
/// );
///
/// fn main() {
///     # VM::init();
///     Class::new("Server", None).define(|klass| {
///         klass.def("start", start);
///     });
/// }
/// ```
///
/// Ruby:
///
/// ```ruby
/// class Server
///   def start(address)
///     default_port = 8080
///
///     port =
///       if address.is_a?(Hash) && address[:port].is_a?(Fixnum)
///         address[:port]
///       else
///         default_port
///       end
///
///     # Start server...
///   end
/// end
/// ```
#[macro_export]
macro_rules! methods {
    (
        $rtself_class: ty,
        $rtself_name: ident,
        $(
            fn $method_name: ident
            ($($arg_name: ident: $arg_type: ty),*) -> $return_type: ty $body: block
            $(,)?
        )*
    ) => {
        $(
            #[allow(unused_mut)]
            pub extern fn $method_name(argc: $crate::types::Argc,
                                       argv: *const $crate::AnyObject,
                                       mut $rtself_name: $rtself_class) -> $return_type {
                let _arguments = $crate::util::parse_arguments(argc, argv);
                let mut _i = 0;

                $(
                    let $arg_name =
                        _arguments
                            .get(_i)
                            .ok_or_else(|| {
                                <$crate::AnyException as $crate::Exception>::new("ArgumentError",
                                    Some(&format!(
                                        "Argument '{}: {}' not found for method '{}'",
                                        stringify!($arg_name),
                                        stringify!($arg_type),
                                        stringify!($method_name)
                                    ))
                                )
                            }).and_then(|argument| {
                                <$crate::AnyObject as $crate::Object>
                                    ::try_convert_to::<$arg_type>(argument)
                            });

                    _i += 1;
                )*

                $body
            }
        )*
    }
}

/// Makes a Rust struct wrappable for Ruby objects.
///
/// **Note:** Currently to be able to use `wrappable_struct!` macro, you should include
/// `lazy_static` crate to the crate you are working on.
///
/// `Cargo.toml`
///
/// ```toml
/// lazy_static = "0.2.1" # the version is not a strict requirement
/// ```
///
/// Crate root `lib.rs` or `main.rs`
///
/// ```ignore
/// #[macro_use]
/// extern crate lazy_static;
/// ```
///
/// # Arguments
///
///  - `$struct_name` is name of the actual Rust struct. This structure has to be public (`pub`).
///
///  - `$wrapper` is a name for the structure which will be created to wrap the `$struct_name`.
///
///     The wrapper will be created automatically by the macro.
///
///  - `$static_name` is a name for a static variable which will contain the wrapper.
///
///     The static variable will be created automatically by the macro.
///
///     This variable has to be passed to `wrap_data()` and `get_data()` functions (see examples).
///
///     Also, these variables describe the structure in general, but not some specific object.
///     So you should pass the same static variable when wrapping/getting data of the same
///     type for different ruby objects.
///
///     For example,
///
///     ```ignore
///     server1.get_data(&*SERVER_WRAPPER);
///     server2.get_data(&*SERVER_WRAPPER); // <-- the same `SERVER_WRAPPER`
///     ```
///
///  - (optional) `mark(data) { ... }` is a block which will be called during the "mark"
///    phase of garbage collection.
///
///    This block must be used if the struct contains any Ruby objects. The objects should
///    be marked with `GC::mark()` to prevent their garbage collection.
///
///    `data` argument will be yielded as a mutable reference to the wrapped struct
///    (`&mut $struct_name`).
///
///    **Notes from the official MRI documentation:**
///
///      - It is not recommended to store Ruby objects in the structs. Try to avoid that
///        if possible.
///
///      - It is not allowed to allocate new Ruby objects in the `mark` function.
///
/// The result of `wrappable_struct!` is:
///
/// ```ignore
/// wrappable_struct!(Server, ServerWrapper, SERVER_WRAPPER);
///
/// // produces
///
/// struct ServerWrapper {
///     // ...
/// }
///
/// pub static ref SERVER_WRAPPER: ServerWrapper<Server> = // ...
/// ```
///
/// # Class
///
/// The class which will be used for wrapping data is `Object` and not `Data`
/// (See [Ruby issue #3072](https://bugs.ruby-lang.org/issues/3072)).
///
/// ```
/// # use rutie::{Class, VM};
/// # VM::init();
/// let data_class = Class::from_existing("Object");
///
/// Class::new("TheNewClass", Some(&data_class));
/// ```
///
/// # Examples
///
/// ## Wrap `Server` structs to `RubyServer` objects
///
/// ```
/// #[macro_use] extern crate rutie;
/// #[macro_use] extern crate lazy_static;
///
/// use rutie::{AnyObject, Class, Fixnum, Object, RString, VM};
///
/// // The structure which we want to wrap
/// pub struct Server {
///     host: String,
///     port: u16,
/// }
///
/// impl Server {
///     fn new(host: String, port: u16) -> Self {
///         Server {
///             host: host,
///             port: port,
///         }
///     }
///
///     fn host(&self) -> &str {
///         &self.host
///     }
///
///     fn port(&self) -> u16 {
///         self.port
///     }
/// }
///
/// wrappable_struct!(Server, ServerWrapper, SERVER_WRAPPER);
///
/// class!(RubyServer);
///
/// methods!(
///     RubyServer,
///     rtself,
///
///     fn ruby_server_new(host: RString, port: Fixnum) -> AnyObject {
///         let server = Server::new(host.unwrap().to_string(),
///                                  port.unwrap().to_i64() as u16);
///
///         Class::from_existing("RubyServer").wrap_data(server, &*SERVER_WRAPPER)
///     }
///
///     fn ruby_server_host() -> RString {
///         let host = rtself.get_data(&*SERVER_WRAPPER).host();
///
///         RString::new_utf8(host)
///     }
///
///     fn ruby_server_port() -> Fixnum {
///         let port = rtself.get_data(&*SERVER_WRAPPER).port();
///
///         Fixnum::new(port as i64)
///     }
/// );
///
/// fn main() {
///     # VM::init();
///     let data_class = Class::from_existing("Object");
///
///     Class::new("RubyServer", Some(&data_class)).define(|klass| {
///         klass.def_self("new", ruby_server_new);
///
///         klass.def("host", ruby_server_host);
///         klass.def("port", ruby_server_port);
///     });
/// }
/// ```
///
/// To use the `RubyServer` class in Ruby:
///
/// ```ruby
/// server = RubyServer.new("127.0.0.1", 3000)
///
/// server.host == "127.0.0.1"
/// server.port == 3000
/// ```
///
/// ## `RustyArray`
///
/// Custom array implementation using a vector which contains `AnyObject`s.
///
/// ```
/// #[macro_use] extern crate rutie;
/// #[macro_use] extern crate lazy_static;
///
/// use std::ops::{Deref, DerefMut};
///
/// use rutie::{AnyObject, Class, Fixnum, GC, NilClass, Object, VM};
///
/// pub struct VectorOfObjects {
///     inner: Vec<AnyObject>,
/// }
///
/// impl VectorOfObjects {
///     fn new() -> Self {
///         VectorOfObjects {
///             inner: Vec::new(),
///         }
///     }
/// }
///
/// impl Deref for VectorOfObjects {
///     type Target = Vec<AnyObject>;
///
///     fn deref(&self) -> &Vec<AnyObject> {
///         &self.inner
///     }
/// }
///
/// impl DerefMut for VectorOfObjects {
///     fn deref_mut(&mut self) -> &mut Vec<AnyObject> {
///         &mut self.inner
///     }
/// }
///
/// wrappable_struct! {
///     VectorOfObjects,
///     VectorOfObjectsWrapper,
///     VECTOR_OF_OBJECTS_WRAPPER,
///
///     // Mark each `AnyObject` element of the `inner` vector to prevent garbage collection.
///     // `data` is a mutable reference to the wrapped data (`&mut VectorOfObjects`).
///     mark(data) {
///         for object in &data.inner {
///             GC::mark(object);
///         }
///     }
/// }
///
/// class!(RustyArray);
///
/// methods! {
///     RustyArray,
///     rtself,
///
///     fn new() -> AnyObject {
///         let vec = VectorOfObjects::new();
///
///         Class::from_existing("RustyArray").wrap_data(vec, &*VECTOR_OF_OBJECTS_WRAPPER)
///     }
///
///     fn push(object: AnyObject) -> NilClass {
///         rtself.get_data_mut(&*VECTOR_OF_OBJECTS_WRAPPER).push(object.unwrap());
///
///         NilClass::new()
///     }
///
///     fn length() -> Fixnum {
///         let length = rtself.get_data(&*VECTOR_OF_OBJECTS_WRAPPER).len() as i64;
///
///         Fixnum::new(length)
///     }
/// }
///
/// fn main() {
///     # VM::init();
///     let data_class = Class::from_existing("Object");
///
///     Class::new("RustyArray", Some(&data_class)).define(|klass| {
///         klass.def_self("new", new);
///
///         klass.def("push", push);
///         klass.def("length", length);
///     });
/// }
/// ```
///
/// To use the `RustyArray` class in Ruby:
///
/// ```ruby
/// array = RustyArray.new
///
/// array.push(1)
/// array.push("string")
/// array.push(:symbol)
///
/// array.length == 3
/// ```
#[macro_export]
macro_rules! wrappable_struct {
    (@mark_function_pointer) => {
        None as Option<extern "C" fn(*mut $crate::types::c_void)>
    };
    // Leading comma is the comma between `$static_name: ident` and `mark` in the main macro rule.
    // Optional comma `$(,)*` is not allowed in the main rule, because it is
    // followed by `$($tail: tt)*`
    (@mark_function_pointer , mark($object: ident) $body: block) => {
        Some(Self::mark as extern "C" fn(*mut $crate::types::c_void))
    };
    (@mark_function_definition $struct_name: ty) => {};
    (@mark_function_definition $struct_name: ty, mark($object: ident) $body: expr) => {
        pub extern "C" fn mark(data: *mut $crate::types::c_void) {
            let mut data = unsafe { (data as *mut $struct_name).as_mut() };

            if let Some(ref mut $object) = data {
                $body
            }
        }
    };
    ($struct_name: ty, $wrapper: ident, $static_name: ident $($tail: tt)*) => {
        pub struct $wrapper<T> {
            data_type: $crate::types::DataType,
            _marker: ::std::marker::PhantomData<T>,
        }

        ::lazy_static::lazy_static! {
            pub static ref $static_name: $wrapper<$struct_name> = $wrapper::new();
        }

        impl<T> $wrapper<T> {
            fn new() -> $wrapper<T> {
                let name = concat!("Rutie/", stringify!($struct_name));
                let name = $crate::util::str_to_cstring(name);
                let reserved_bytes: [*mut $crate::types::c_void; 2] = [::std::ptr::null_mut(); 2];

                let dmark = wrappable_struct!(@mark_function_pointer $($tail)*);

                let data_type = $crate::types::DataType {
                    wrap_struct_name: name.into_raw(),
                    parent: ::std::ptr::null(),
                    data: ::std::ptr::null_mut(),
                    flags: $crate::types::Value::from(0),

                    function: $crate::types::DataTypeFunction {
                        dmark: dmark,
                        dfree: Some($crate::typed_data::free::<T>),
                        dsize: None,
                        reserved: reserved_bytes,
                    },
                };

                $wrapper {
                    data_type: data_type,
                    _marker: ::std::marker::PhantomData,
                }
            }

            wrappable_struct!(@mark_function_definition $struct_name $($tail)*);
        }

        unsafe impl<T> Sync for $wrapper<T> {}

        // Set constraint to be able to wrap and get data only for type `T`
        impl<T> $crate::typed_data::DataTypeWrapper<T> for $wrapper<T> {
            fn data_type(&self) -> &$crate::types::DataType {
                &self.data_type
            }
        }
    };
}

/// eval(string [, binding [, filename [,lineno]]]) → obj
///
/// # Examples
/// ```
/// #[macro_use]
/// extern crate rutie;
/// use rutie::{Object, Integer, Binding, VM};
///
/// fn main() {
///     # VM::init();
///
///     let binding = eval!("asdf = 1; binding").unwrap().
///       try_convert_to::<Binding>().unwrap();
///
///     let result = eval!("asdf", binding).unwrap();
///
///     match result.try_convert_to::<Integer>() {
///         Ok(v) => assert_eq!(1, v.to_i64()),
///         Err(_) => unreachable!(),
///     }
/// }
/// ```
#[macro_export]
macro_rules! eval {
    ($string_arg:expr) => {{ $crate::VM::eval($string_arg) }};
    ($string_arg:expr, $binding_arg:expr) => {{
        let eval_str: $crate::AnyObject = $crate::RString::from($string_arg).into();
        let bndng: $crate::AnyObject = $binding_arg.into();
        let arguments = &[eval_str, bndng];

        $crate::Class::from_existing("Kernel").protect_send("eval", arguments)
    }};
    ($string_arg:expr, $binding_arg:expr, $filename:expr) => {{
        let eval_str: $crate::AnyObject = $crate::RString::from($string_arg).into();
        let bndng: $crate::AnyObject = $binding_arg.into();
        let filename: $crate::AnyObject = $crate::RString::from($filename).into();
        let arguments = &[eval_str, bndng, filename];

        $crate::Class::from_existing("Kernel").protect_send("eval", arguments)
    }};
    ($string_arg:expr, $binding_arg:expr, $filename:expr, $linenumber:expr) => {{
        let eval_str: $crate::AnyObject = $crate::RString::from($string_arg).into();
        let bndng: $crate::AnyObject = $binding_arg.into();
        let filename: $crate::AnyObject = $crate::RString::from($filename).into();
        let linenumber: $crate::AnyObject = $crate::Integer::from($linenumber as i64).into();
        let arguments = &[eval_str, bndng, filename, linenumber];

        $crate::Class::from_existing("Kernel").protect_send("eval", arguments)
    }};
}

/// Define modules, classes, and methods using a `ruby_define!` macros.
///
/// Example
///
/// ```rust
/// pub extern "C" fn init() {
///     let mut wasmer_module = Module::from_existing("Wasmer");
///
///     wasmer_module.const_set("VERSION", &RString::new_utf8(env!("CARGO_PKG_VERSION")));
///
///     ruby_define! {
///         in wasmer_module
///             class (store::ruby_store) Store {
///                 def_self (new) "new";
///             };
///
///             class (module::ruby_module) Module {
///                 def_self (validate) "validate";
///                 def_self (new) "new";
///                 def (set_name) "name=";
///                 def (get_name) "name";
///                 def (exports) "exports";
///                 def (imports) "imports";
///                 def (custom_sections) "custom_sections";
///                 def (serialize) "serialize";
///                 def_self (deserialize) "deserialize";
///             };
///
///             class (instance::ruby_instance) Instance {
///                 def_self (new) "new";
///                 def (exports) "exports";
///             };
///
///             class (exports::ruby_exports, exports::ruby_exports_extra) Exports {
///                 def (respond_to_missing) "respond_to_missing?";
///                 def (method_missing) "method_missing";
///                 def (length) "length";
///             };
///
///             class (import_object::ruby_importobject) ImportObject {
///                 def_self (new) "new";
///                 def (contains_namespace) "contains_namespace?";
///                 def (register) "register";
///             };
///
///             class (externals::function::ruby_function, externals::function::ruby_function_extra) Function {
///                 def_self (new) "new";
///                 def (call) "call";
///                 def (r#type) "type";
///             };
///
///             class (externals::memory::ruby_memory) Memory {
///                 def_self (new) "new";
///                 def (r#type) "type";
///                 def (size) "size";
///                 def (data_size) "data_size";
///                 def (grow) "grow";
///                 def (uint8_view) "uint8_view";
///                 def (int8_view) "int8_view";
///                 def (uint16_view) "uint16_view";
///                 def (int16_view) "int16_view";
///                 def (uint32_view) "uint32_view";
///                 def (int32_view) "int32_view";
///             };
///
///             class (memory::views::ruby_uint8array) Uint8Array
///             include Enumerable
///             {
///                 @const BYTES_PER_ELEMENT = Integer::from(Uint8Array::BYTES_PER_ELEMENT);
///                 def (length) "length";
///                 def (set) "[]=";
///                 def (get) "[]";
///                 def (each) "each";
///             };
///
///             class (memory::views::ruby_int8array) Int8Array
///             include Enumerable
///             {
///                 @const BYTES_PER_ELEMENT = Integer::from(Int8Array::BYTES_PER_ELEMENT);
///                 def (length) "length";
///                 def (set) "[]=";
///                 def (get) "[]";
///                 def (each) "each";
///             };
///
///             class (memory::views::ruby_uint16array) Uint16Array
///             include Enumerable
///             {
///                 @const BYTES_PER_ELEMENT = Integer::from(Uint16Array::BYTES_PER_ELEMENT);
///                 def (length) "length";
///                 def (set) "[]=";
///                 def (get) "[]";
///                 def (each) "each";
///             };
///
///             class (memory::views::ruby_int16array) Int16Array
///             include Enumerable
///             {
///                 @const BYTES_PER_ELEMENT = Integer::from(Int16Array::BYTES_PER_ELEMENT);
///                 def (length) "length";
///                 def (set) "[]=";
///                 def (get) "[]";
///                 def (each) "each";
///             };
///
///             class (memory::views::ruby_uint32array) Uint32Array
///             include Enumerable
///             {
///                 @const BYTES_PER_ELEMENT = Integer::from(Uint32Array::BYTES_PER_ELEMENT);
///                 def (length) "length";
///                 def (set) "[]=";
///                 def (get) "[]";
///                 def (each) "each";
///             };
///
///             class (memory::views::ruby_int32array) Int32Array
///             include Enumerable
///             {
///                 @const BYTES_PER_ELEMENT = Integer::from(Int32Array::BYTES_PER_ELEMENT);
///                 def (length) "length";
///                 def (set) "[]=";
///                 def (get) "[]";
///                 def (each) "each";
///             };
///
///             class (externals::global::ruby_global) Global {
///                 def_self (new) "new";
///                 def (mutable) "mutable?";
///                 def (get_value) "value";
///                 def (set_value) "value=";
///                 def (r#type) "type";
///             };
///
///             class (externals::table::ruby_table) Table {
///                 def_self (new) "new";
///             };
///
///             class (types) Type {
///                 @const I32 = Integer::new(1);
///                 @const I64 = Integer::new(2);
///                 @const F32 = Integer::new(3);
///                 @const F64 = Integer::new(4);
///                 @const V128 = Integer::new(5);
///                 @const EXTERN_REF = Integer::new(6);
///                 @const FUNC_REF = Integer::new(7);
///             };
///
///             class (types::ruby_functiontype) FunctionType {
///                 def_self (new) "new";
///                 def (params) "params";
///                 def (results) "results";
///             };
///
///             class (types::ruby_memorytype) MemoryType {
///                 def_self (new) "new";
///                 def (minimum) "minimum";
///                 def (maximum) "maximum";
///                 def (shared) "shared?";
///             };
///
///             class (types::ruby_globaltype) GlobalType {
///                 def_self (new) "new";
///                 def (r#type) "type";
///                 def (mutable) "mutable?";
///             };
///
///             class (types::ruby_tabletype) TableType {
///                 def_self (new) "new";
///                 def (r#type) "type";
///                 def (minimum) "minimum";
///                 def (maximum) "maximum";
///             };
///
///             class (types::ruby_exporttype) ExportType {
///                 def_self (new) "new";
///                 def (name) "name";
///                 def (r#type) "type";
///             };
///
///             class (types::ruby_importtype) ImportType {
///                 def_self (new) "new";
///                 def (module) "module";
///                 def (name) "name";
///                 def (r#type) "type";
///             };
///
///             class (values::ruby_value) Value {
///                 def_self (i32) "i32";
///                 def_self (i64) "i64";
///                 def_self (f32) "f32";
///                 def_self (f64) "f64";
///             };
///
///             function (wat::wat2wasm) "wat2wasm";
///             function (wat::wasm2wat) "wasm2wat";
///     };
///
///     let mut wasmer_wasi_module = wasmer_module.define_nested_module("Wasi");
///
///     ruby_define! {
///         in wasmer_wasi_module
///             class (wasi) Version {
///                 @const LATEST = Integer::new(1);
///                 @const SNAPSHOT0 = Integer::new(2);
///                 @const SNAPSHOT1 = Integer::new(3);
///             };
///
///             class (wasi::ruby_statebuilder) StateBuilder {
///                 def_self (new) "new";
///                 def (arguments) "arguments";
///                 def (argument) "argument";
///                 def (environments) "environments";
///                 def (environment) "environment";
///                 def (preopen_directories) "preopen_directories";
///                 def (preopen_directory) "preopen_directory";
///                 def (map_directories) "map_directories";
///                 def (map_directory) "map_directory";
///                 def (finalize) "finalize";
///             };
///
///             class (wasi::ruby_environment) Environment {
///                 def (generate_import_object) "generate_import_object";
///             };
///
///             function (wasi::get_version) "get_version";
///     };
/// }
/// ```
#[macro_export]
macro_rules! ruby_define {
    (in $module:ident
     $( class ( $( $class_rust_module:path ),+ ) $class_name:ident
        $( include $include_name:ident )*
        {
           $( @const $constant_name:ident = $constant_value:expr; )*
           $( $ruby_definition:ident ($method_rust_name:ident) $method_name:expr; )*
        };
     )*

     $( function ( $function_rust_name:path ) $function_name:expr; )*
    ) => {
        $(
            {
                let data_class = Class::from_existing("Object");

                $module
                    .define_nested_class(stringify!($class_name), Some(&data_class))
                    .define(|this| {
                        $(
                            #[allow(unused_imports)]
                            use $class_rust_module::*;
                        )+

                        $(
                            this.$ruby_definition($method_name, $method_rust_name);
                        )*

                        $(
                            this.const_set(stringify!($constant_name), &$constant_value);
                        )*
                    })
                    $( .include( stringify!($include_name) ) )*
                    ;
            }
        )*

        $( $module.define_module_function($function_name, $function_rust_name); )*
    }
}
