use crate::enum_info::EnumInfo;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

pub fn generate(info: EnumInfo) -> TokenStream {
    let errcode = quote!(errcode);
    let internal = quote!(#errcode::__macro_export);
    let core = quote!(#errcode::__macro_export::core);

    let ty = &info.name;
    let ty_name = ty.to_string();

    let error_code_info_ident: Vec<_> = (0..info.variants.len())
        .map(|x| Ident::new(&format!("ERROR_CODE_INFO_{x}"), Span::call_site()))
        .collect();
    let static_info_ident: Vec<_> = (0..info.variants.len())
        .map(|x| Ident::new(&format!("STATIC_INFO_{x}"), Span::call_site()))
        .collect();

    let ids: Vec<_> = info.variants.iter().map(|x| x.repr).collect();
    let variant_names: Vec<_> = info.variants.iter().map(|x| x.name.to_string()).collect();
    let variant: Vec<_> = info.variants.iter().map(|x| &x.name).collect();
    let message_data: Vec<_> = info
        .variants
        .iter()
        .map(|x| match &x.message {
            None => quote! { #internal::None },
            Some(msg) => quote! { #internal::Some(#msg) },
        })
        .collect();

    quote! {
        #[automatically_derived]
        const _: () = {
            const TYPE_ID: #core::any::TypeId = #core::any::TypeId::of::<#ty>();
            #(
                static #error_code_info_ident: #internal::ErrorCodeInfo = #internal::ErrorCodeInfo {
                    tid: TYPE_ID,
                    value: #ids,
                    type_name: #ty_name,
                    variant_name: #variant_names,
                    message: #message_data,
                };
                static #static_info_ident: #internal::ErrorInfoImpl =
                    #internal::wrap_code(&#error_code_info_ident);
            )*

            pub struct ConstHelperType;
            impl ConstHelperType {
                pub const fn info(&self, value: #ty) -> &'static #internal::ErrorCodeInfo {
                    match value {
                        #(#ty::#variant => &#error_code_info_ident,)*
                    }
                }
            }

            impl #internal::ErrorCodePrivate for #ty {
                type ConstHelper = ConstHelperType;
                const CONST_HELPER_INSTANCE: ConstHelperType = ConstHelperType;

                fn error_source(self) -> &'static #internal::ErrorInfoImpl {
                    match self {
                        #(#ty::#variant => &#static_info_ident,)*
                    }
                }
                fn is_value(self, value: u32) -> bool {
                    match value {
                        #(#ids => #core::matches!(self, #ty::#variant),)*
                        _ => false,
                    }
                }
                fn from_value(value: u32) -> Self {
                    match value {
                        #(#ids => #ty::#variant,)*
                        _ => #core::panic!("unknown value: {value}"),
                    }
                }
            }
            impl #errcode::ErrorCode for #ty {}
        };
    }
}
