use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Error};

#[proc_macro_derive(SpinFactors)]
pub fn derive_factors(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_factors(&input)
        .unwrap_or_else(|err| err.into_compile_error())
        .into()
}

#[allow(non_snake_case)]
fn expand_factors(input: &DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let vis = &input.vis;

    let builders_name = format_ident!("{name}_Builders");
    let data_name = format_ident!("{name}_Data");

    if !input.generics.params.is_empty() {
        return Err(Error::new_spanned(
            input,
            "cannot derive Factors for generic structs",
        ));
    }

    // Get struct fields
    let fields = match &input.data {
        Data::Struct(struct_data) => &struct_data.fields,
        _ => {
            return Err(Error::new_spanned(
                input,
                "can only derive Factors for structs",
            ))
        }
    };
    let mut factor_names = Vec::with_capacity(fields.len());
    let mut factor_types = Vec::with_capacity(fields.len());
    for field in fields.iter() {
        factor_names.push(
            field
                .ident
                .as_ref()
                .ok_or_else(|| Error::new_spanned(input, "tuple structs are not supported"))?,
        );
        factor_types.push(&field.ty);
    }

    let factors_crate = format_ident!("spin_factor");
    let factors_path = quote!(::#factors_crate);
    let Factor = quote!(#factors_path::Factor);
    let Result = quote!(#factors_path::Result);
    let wasmtime = quote!(#factors_path::wasmtime);
    let TypeId = quote!(::std::any::TypeId);

    Ok(quote! {
        impl #name {
            pub fn new(
                #( #factor_names: #factor_types, )*
            ) -> Self {
                Self {
                    #( #factor_names, )*
                }
            }

            pub fn add_to_linker(
                linker: &mut #wasmtime::component::Linker<#data_name>
            ) -> #Result<()> {
                #(
                    <#factor_types as #Factor>::add_to_linker::<#name>(
                        linker, |data| &mut data.#factor_names)?;
                )*
                Ok(())
            }

            pub fn add_to_module_linker(
                linker: &mut #wasmtime::Linker<#data_name>
            ) -> #Result<()> {
                #(
                    <#factor_types as #factors_path::Factor>::add_to_module_linker::<#name>(
                        linker, |data| &mut data.#factor_names)?;
                )*
                Ok(())
            }

            pub fn build_data(&self) -> #Result<#data_name> {
                let mut builders = #builders_name {
                    #( #factor_names: None, )*
                };
                #(
                    builders.#factor_names = Some(
                        <#factor_types as #Factor>::InstanceBuilder::prepare::<#name>(
                            &self.#factor_names,
                            #factors_path::PrepareContext::new(&mut builders),
                        )?
                    );
                )*
                Ok(#data_name {
                    #(
                        #factor_names: #factors_path::InstanceBuilder::<#factor_types>::build(
                            builders.#factor_names.unwrap()
                        )?,
                    )*
                })
            }
        }

        impl #factors_path::SpinFactors for #name {
            type InstanceBuilders = #builders_name;
            type InstanceData = #data_name;

            fn data_getter<T: #factors_path::Factor>(
            ) -> Option<for<'a> fn(&'a mut Self::InstanceData) -> &'a mut T::InstanceData> {
                let type_id = #TypeId::of::<T>();
                #(
                    fn #factor_names(data: &mut #data_name) -> &mut <#factor_types as #factors_path::Factor>::InstanceData {
                        &mut data.#factor_names
                    }
                    if type_id == #TypeId::of::<#factor_types>() {
                        let ptr = #factor_names as *const ();
                        return Some(unsafe { std::mem::transmute(ptr) });
                    }
                )*
                None
            }

            fn builder_mut<T: #Factor>(
                builders: &mut Self::InstanceBuilders,
            ) -> Option<Option<&mut T::InstanceBuilder>> {
                let type_id = #TypeId::of::<T>();
                #(
                    if type_id == #TypeId::of::<#factor_types>() {
                        return Some(
                            builders.#factor_names.as_mut().map(|builder| {
                                unsafe { std::mem::transmute(builder) }
                            })
                        );
                    }
                )*
                None
            }
        }

        #vis struct #builders_name {
            #(
                #factor_names: Option<<#factor_types as #Factor>::InstanceBuilder>,
            )*
        }

        #vis struct #data_name {
            #(
                #factor_names: <#factor_types as #Factor>::InstanceData,
            )*
        }
    })
}
