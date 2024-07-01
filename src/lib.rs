use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

fn generate_generics_list(
    how_many: usize,
    pos_to_skip: usize,
) -> impl Iterator<Item = proc_macro2::TokenStream> {
    (0..how_many)
        .filter(move |&index| index != pos_to_skip)
        .map(|index| {
            let generic_name = quote::format_ident!("T{}", index);
            quote! {
                #generic_name
            }
        })
}

fn current_builder_tags(
    how_many: usize,
    pos_of_interest: usize,
) -> impl Iterator<Item = proc_macro2::TokenStream> {
    (0..how_many).map(move |index| {
        if pos_of_interest == index {
            quote! {
                Zero
            }
        } else {
            let generic_name = quote::format_ident!("T{}", index);
            quote! {
                #generic_name
            }
        }
    })
}

fn next_builder_tags(
    how_many: usize,
    pos_of_interest: usize,
) -> impl Iterator<Item = proc_macro2::TokenStream> {
    (0..how_many).map(move |index| {
        if pos_of_interest == index {
            quote! {
                One
            }
        } else {
            let generic_name = quote::format_ident!("T{}", index);
            quote! {
                #generic_name
            }
        }
    })
}

#[proc_macro_derive(Builder)]
pub fn derive_builder(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let syn::Data::Struct(data_structure) = ast.data else {
        panic!("Builder derive only works with structs")
    };
    let syn::Fields::Named(fields) = data_structure.fields else {
        panic!("Builder derive only works with named fields")
    };
    let struct_name = &ast.ident;
    let builder_fields = fields.named.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;
        quote! {
            #field_name: Option<#field_type>,
        }
    });

    let builder_initial_tags = fields.named.iter().map(|_| {
        quote! {
            Zero
        }
    });

    let builder_final_tags = fields.named.iter().map(|_| {
        quote! {
            One
        }
    });

    let builder_unwrap_fields = fields.named.iter().map(|field| {
        let field_name = &field.ident;
        quote! {
            #field_name: self.#field_name.unwrap(),
        }
    });

    let builder_impls = fields.named.iter().enumerate().map(|(index, field)| {
        let method_name = quote::format_ident!("with_{}", field.ident.as_ref().unwrap());
        let field_name = &field.ident;
        let field_type = &field.ty;
        let generics_list = generate_generics_list(fields.named.len(), index);
        let current_builder_tags = current_builder_tags(fields.named.len(), index);
        let next_builder_tags = next_builder_tags(fields.named.len(), index);
        quote! {
            impl<#(#generics_list),*> Builder<(#(#current_builder_tags),*)> {
                pub fn #method_name(mut self, #field_name: #field_type) -> Builder<(#(#next_builder_tags),*)> {
                    self.#field_name = Some(#field_name);
                    unsafe {std::mem::transmute(self)}
                }
            }
        }
    });

    quote! {
        #[derive(Default)]
        struct Zero;

        #[derive(Default)]
        struct One;

        #[derive(Default)]
        struct Builder<Step> {
            #(#builder_fields)*
            _step: std::marker::PhantomData<Step>,
        }

        impl #struct_name {
            pub fn builder() -> Builder<(#(#builder_initial_tags),*)> {
                Builder::default()
            }
        }

        impl Builder<(#(#builder_final_tags),*)> {
            pub fn build(self) -> #struct_name {
                #struct_name {
                    #(#builder_unwrap_fields)*
                }
            }
        }

        #(#builder_impls)*
    }
    .into()
}
