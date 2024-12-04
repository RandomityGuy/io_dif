extern crate proc_macro;
extern crate quote;
extern crate syn;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, GenericArgument, PathArguments, Type};

// #[derive(Readable)] implements io::Readable<T> for a struct T whose body
// reads (in sequence) all the members of T and returns Ok(T {members})
#[proc_macro_derive(Readable)]
pub fn trivial_read_fn(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(item as DeriveInput);

    let name = &ast.ident;
    let fields = read_generate_fields(&ast.data);

    let expanded = quote! {
        impl Readable<#name> for #name {
            fn read(from: &mut dyn Buf, version: &mut Version) -> DifResult<#name> {
                Ok(#name {
                    #fields
                })
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

// #[derive(Writable)] implements io::Writable<T> for a struct T whose body
// writes (in sequence) all the members of T and returns Ok
#[proc_macro_derive(Writable)]
pub fn trivial_write_fn(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(item as DeriveInput);

    let name = &ast.ident;
    let fields = write_generate_fields(&ast.data);

    let expanded = quote! {
        impl Writable<#name> for #name {
            fn write(&self, to: &mut dyn BufMut, version: &Version) -> DifResult<()> {
                #fields
                Ok(())
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

// Take all the fields in a struct and generate `field: FType::read(from, version)?`
// for each field.
fn read_generate_fields(data: &Data) -> TokenStream {
    match data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    let field_reads = fields.named.iter().map(|f| {
                        let name = &f.ident;
                        // Generics need an extra :: so split that off into its own function
                        let ftype = type_turbofish(&f.ty);

                        quote! {
                            #name: #ftype::read(from, version)?
                        }
                    });
                    quote! {
                        #(#field_reads, )*
                    }
                }
                Fields::Unnamed(_) | Fields::Unit => unimplemented!(),
            }
        }
        Data::Enum(_) | Data::Union(_) => unimplemented!(),
    }
}

// Take all the fields in a struct and generate `self.field.write(to, version)?`
// for each field.
fn write_generate_fields(data: &Data) -> TokenStream {
    match data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    let field_writes = fields.named.iter().map(|f| {
                        let name = &f.ident;
                        quote! {
                            self.#name.write(to, version)?
                        }
                    });
                    quote! {
                        #(#field_writes;)*
                    }
                }
                Fields::Unnamed(_) | Fields::Unit => unimplemented!(),
            }
        }
        Data::Enum(_) | Data::Union(_) => unimplemented!(),
    }
}

// We can't do Vec<u16>::read, so this function adds an extra :: before the <> to make
// this syntactically valid
fn type_turbofish(t: &Type) -> TokenStream {
    match t {
        // Path is basically all "normal" types
        Type::Path(typepath) => {
            // Take all the segmented parts of the path and turbofish them all
            // (I assume we don't have to, but might as well)
            let mut segments = typepath
                .path
                .segments
                .iter()
                .map(|segment| {
                    let ident = &segment.ident;

                    match &segment.arguments {
                        PathArguments::None => {
                            // Normal (non-generic) type names are just the name
                            quote! { #ident }
                        }
                        PathArguments::AngleBracketed(args) => {
                            // Eg Vec<String>, we need to stringify all the type args so we
                            // can put them back into tokens
                            let mut types = args
                                .args
                                .iter()
                                .map(|genarg| match genarg {
                                    GenericArgument::Type(ty) => type_turbofish(ty),
                                    _ => unimplemented!(),
                                })
                                .collect::<Vec<_>>();

                            // There's probably a better way to do this
                            let first = types.remove(0);
                            quote! { #ident :: < #first #(, #types)* >}
                        }
                        PathArguments::Parenthesized(_) => unimplemented!(),
                    }
                })
                .collect::<Vec<_>>();

            // There's probably a better way to do this
            let first = segments.remove(0);
            quote! { #first #( :: #segments)* }
        }
        _ => {
            quote! { #t }
        }
    }
}
