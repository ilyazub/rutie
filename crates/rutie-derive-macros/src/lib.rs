mod class;
mod function;
mod methods;

/// Example
///
/// ```rust
/// #[rubyclass(module = "Wasmer")]
/// pub struct Instance {
///     _inner: wasmer::Instance,
///     exports: AnyObject,
/// }
///
/// #[rubymethods]
/// impl Instance {
///     pub fn new(module: &Module, import_object: &AnyObject) -> RubyResult<AnyObject> {
///         let module = module.inner();
///
///         let instance = if import_object.is_nil() {
///             wasmer::Instance::new(&module, &wasmer::imports! {})
///         } else {
///             wasmer::Instance::new(
///                 &module,
///                 import_object
///                     .try_convert_to::<RubyImportObject>()?
///                     .upcast()
///                     .inner(),
///             )
///         };
///
///         let instance = instance.map_err(to_ruby_err::<RuntimeError, _>)?;
///         let exports = Exports::ruby_new(Exports::new(instance.exports.clone()));
///
///         Ok(Instance::ruby_new(Instance {
///             _inner: instance,
///             exports,
///         }))
///     }
///
///     pub fn exports(&self) -> RubyResult<AnyObject> {
///         Ok(self.exports.clone())
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn rubyclass(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    class::entry(attr, input)
}

#[proc_macro_attribute]
pub fn rubymethods(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    methods::entry(attr, input)
}

#[proc_macro_attribute]
pub fn rubyfunction(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    function::entry(attr, input)
}
