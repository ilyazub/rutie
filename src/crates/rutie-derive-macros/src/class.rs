use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, AttributeArgs, Data, DataStruct, DeriveInput, Generics, Ident, Lit, Meta,
    MetaNameValue, NestedMeta,
};

pub fn entry(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let cloned_input = input.clone();
    let derive_input = parse_macro_input!(cloned_input as DeriveInput);
    let arguments = parse_macro_input!(attr as AttributeArgs);

    if arguments.is_empty() {
        panic!("The `rubyclass` procedural macro must have a `module` argument, e.g. `#[rubyclass(module = \"foo\")`");
    }

    let mut ruby_module = None;

    for argument in arguments.iter() {
        match argument {
            NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                path: name,
                lit: Lit::Str(value),
                ..
            })) if name.is_ident("module") => {
                ruby_module = Some(value.value());
            }

            argument => panic!(
                "Unexpected argument `{:?}` from the `rubyclass` procedural macro",
                argument
            ),
        }
    }

    let derived = match derive_input.data {
        Data::Struct(ref struct_data) => derive_for_struct(
            &derive_input.ident,
            struct_data,
            &derive_input.generics,
            ruby_module.expect("The `module` argument of the `rubyclass` procedural macro is missing, e.g. `#[rubyclass(module = \"foo\")]`"),
        ),

        Data::Enum(_) => panic!("enums are not yet supported"),

        Data::Union(_) => panic!("unions are not yet supported"),
    };

    let input = TokenStream::from(input);

    (quote! { #input #derived }).into()
}

fn derive_for_struct(
    struct_name: &Ident,
    _data: &DataStruct,
    generics: &Generics,
    ruby_module: String,
) -> proc_macro2::TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let span = struct_name.span();
    let wrapper_struct_name = Ident::new(&format!("{}Wrapper", struct_name), span);
    let wrapper_const_name = Ident::new(
        &format!("{}_WRAPPER", struct_name.to_string().to_uppercase()),
        span,
    );
    let ruby_struct_name = Ident::new(&format!("Ruby{}", struct_name), span);

    quote! {
        use rutie::{wrappable_struct, typed_data::DataTypeWrapper};

        // Create the `XXXWrapper` wrapper class.
        wrappable_struct!(#struct_name, #wrapper_struct_name, #wrapper_const_name);

        // Implement the `RubyXXX` class.
        rutie::class!(#ruby_struct_name);

        // Implement `rutie::VerifiedObject`.
        impl #impl_generics rutie::VerifiedObject for #ruby_struct_name #ty_generics
        #where_clause
        {
            fn is_correct_type<T>(object: &T) -> bool
            where T: rutie::Object
            {
                object.class() == rutie::Module::from_existing(#ruby_module).get_nested_class(stringify!(#struct_name))
            }

            fn error_message() -> &'static str {
                concat!("Error converting to `", stringify!(#struct_name), "`")
            }
        }

        impl rutie_derive::ClassInfo for #struct_name {
            type Class = #struct_name;
            type RubyClass = #ruby_struct_name;
        }

        impl rutie_derive::ClassInfo for #ruby_struct_name {
            type Class = #struct_name;
            type RubyClass = #ruby_struct_name;
        }

        impl #impl_generics rutie_derive::UpcastRubyClass<#struct_name #ty_generics> for #ruby_struct_name
        #where_clause
        {
            fn upcast(&self) -> &#struct_name {
                rutie::Object::get_data(self, &*#wrapper_const_name)
            }

            fn upcast_mut(&mut self) -> &mut #struct_name {
                rutie::Object::get_data_mut(self, &*#wrapper_const_name)
            }
        }

        // Custom logic to support `wrap` and `uwnrap`.
        impl #impl_generics #struct_name #ty_generics
        #where_clause
        {
            pub(crate) fn ruby_new(this: Self) -> rutie::AnyObject {
                rutie::Module::from_existing(#ruby_module)
                    .get_nested_class(stringify!(#struct_name))
                    .wrap_data(this, &*#wrapper_const_name)
            }
        }
    }
}
