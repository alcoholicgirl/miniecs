use lazy_static::lazy_static;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Mutex;
use syn::parse::*;
use syn::*;

lazy_static! {
    // static ref COMPONENTS: Mutex<HashSet<&'static str>> = Mutex::new(HashSet::new());
    static ref HASHER: Mutex<DefaultHasher> = Mutex::new(DefaultHasher::new());
    static ref COMPONENT_COUNT : Mutex<usize> = Mutex::new(0);
}

#[proc_macro_derive(Component)]
pub fn component_derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    let output: TokenStream = {
        let name = input.ident;
        let generics = input.generics;
        let (_impl, _ty, _where) = generics.split_for_impl();

        if !generics.params.is_empty() {
            panic!("struct with generics cannot be made into a component");
        }

        let ident = {
            let mut hasher = HASHER.try_lock().unwrap();
            let mut count = COMPONENT_COUNT.try_lock().unwrap();
            let n = name.to_string();
            n.hash(&mut *hasher);
            count.hash(&mut *hasher);
            *count += 1;
            let id = hasher.finish();
            id as usize
        };

        quote! {
            impl miniecs::component::ComponentType for #name {
                const IDENTIFIER : usize = #ident;
                const MUTABLE : bool = false;

                type PROTOTYPE = #name;
            }

            impl miniecs::component::ComponentType for &'static #name {
                const IDENTIFIER : usize = #ident;
                const MUTABLE : bool = false;

                type PROTOTYPE = #name;
            }

            impl miniecs::component::ComponentType for &'static mut #name {
                const IDENTIFIER : usize = #ident;
                const MUTABLE : bool = true;

                type PROTOTYPE = #name;
            }

            impl miniecs::component::ComponentInstance for #name {
                fn get_component_id(&self) -> usize {
                    #ident
                }
            }

        }
        .into()
    };
    output
}

struct AllTuples {
    macro_ident: syn::Ident,
    total: usize,
    ident: syn::Ident,
}

impl Parse for AllTuples {
    fn parse(input: ParseStream) -> Result<Self> {
        let macro_ident = input.parse::<syn::Ident>()?;
        input.parse::<token::Comma>()?;
        let total = input.parse::<LitInt>()?.base10_parse()?;
        input.parse::<token::Comma>()?;
        let ident = input.parse::<syn::Ident>()?;

        Ok(AllTuples {
            macro_ident,
            total,
            ident,
        })
    }
}

#[proc_macro]
pub fn all_tuples(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as AllTuples);
    let len = input.total;
    let mut ident_tuples = vec![];
    for i in 0..len {
        let ident = format_ident!("{}{}", input.ident, i);
        ident_tuples.push(quote! {
            #ident
        });
    }

    let macro_ident = &input.macro_ident;
    let invocations = (0..len).map(|i| {
        let ident_tuples = &ident_tuples[..=i];
        quote! {
            #macro_ident!(#(#ident_tuples),*);
        }
    });
    TokenStream::from(quote! {
        #(
            #invocations
        )*
    })
}
