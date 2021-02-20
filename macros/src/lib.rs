#![deny(rust_2018_compatibility)]
#![deny(rust_2018_idioms)]
#![deny(warnings)]
#![recursion_limit = "128"]

extern crate proc_macro;

use proc_macro::TokenStream;

use proc_macro2::Span;
use quote::quote;
use syn::{parse, parse_macro_input, FnArg, ItemFn, ItemStatic, ReturnType, Type};

/// Creates a singleton allocator from a static that implements `GlobalAlloc`
#[proc_macro_attribute]
pub fn allocator(args: TokenStream, input: TokenStream) -> TokenStream {
    if !args.is_empty() {
        return parse::Error::new(Span::call_site(), "`#[allocator]` takes no arguments")
            .to_compile_error()
            .into();
    }

    let item = parse_macro_input!(input as ItemStatic);

    let attrs = &item.attrs;
    let expr = &item.expr;
    let ident = &item.ident;
    let ty = &item.ty;
    let vis = &item.vis;
    quote!(
        #vis struct #ident;

        impl core::ops::Deref for #ident {
            type Target = #ty;

            fn deref(&self) -> &#ty {
                #(#attrs)*
                static #ident: #ty = #expr;

                &#ident
            }
        }

        unsafe impl alloc_many::Alloc for #ident {
            #[inline(always)]
            unsafe fn alloc(layout: Layout) -> *mut u8 {
                <#ty as core::alloc::GlobalAlloc>::alloc(&#ident, layout)
            }

            #[inline(always)]
            unsafe fn dealloc(ptr: *mut u8, layout: Layout) {
                <#ty as core::alloc::GlobalAlloc>::dealloc(&#ident, ptr, layout)
            }

            #[inline(always)]
            unsafe fn alloc_zeroed(layout: Layout) -> *mut u8 {
                <#ty as core::alloc::GlobalAlloc>::alloc_zeroed(&#ident, layout)
            }

            #[inline(always)]
            unsafe fn realloc(ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
                <#ty as core::alloc::GlobalAlloc>::realloc(&#ident, ptr, layout, new_size)
            }
        }
    )
    .into()
}

/// Declares the `Main` allocator
#[proc_macro_attribute]
pub fn main_allocator(args: TokenStream, input: TokenStream) -> TokenStream {
    if !args.is_empty() {
        return parse::Error::new(Span::call_site(), "`#[main_allocator]` takes no arguments")
            .to_compile_error()
            .into();
    }

    let item = parse_macro_input!(input as ItemStatic);

    let attrs = &item.attrs;
    let expr = &item.expr;
    let ident = &item.ident;
    let ty = &item.ty;
    let vis = &item.vis;
    quote!(
        #(#attrs)*
        #vis static #ident: #ty = #expr;

        #[no_mangle]
        static ALLOC_MANY_MAIN: &'static (dyn core::alloc::GlobalAlloc + Sync) = &#ident;
    )
    .into()
}

/// Defines the OOM (Out Of Memory) handler
#[proc_macro_attribute]
pub fn oom(args: TokenStream, input: TokenStream) -> TokenStream {
    if !args.is_empty() {
        return parse::Error::new(Span::call_site(), "`#[oom]` takes no arguments")
            .to_compile_error()
            .into();
    }

    let item = parse_macro_input!(input as ItemFn);

    let arg = item.decl.inputs.iter().next().and_then(|arg| {
        if let FnArg::Captured(arg) = arg {
            Some(arg)
        } else {
            None
        }
    });
    if item.constness.is_some()
        || item.asyncness.is_some()
        || item.abi.is_some()
        || !item.decl.generics.params.is_empty()
        || item.decl.generics.where_clause.is_some()
        || item.decl.inputs.len() != 1
        || arg.is_none()
        || item.decl.variadic.is_some()
        || !is_bottom(&item.decl.output)
    {
        return parse::Error::new(
            Span::call_site(),
            "`#[oom]` must have signature `fn(core::alloc::Layout) -> !`",
        )
        .to_compile_error()
        .into();
    }

    let arg = arg.expect("UNREACHABLE");
    let ident = &item.ident;
    let block = &item.block;
    quote!(
        #[export_name = "alloc_many_oom"]
        fn #ident(#arg) -> ! {
            let _: fn(core::alloc::Layout) -> ! = #ident;

            #block
        }

    )
    .into()
}

fn is_bottom(ty: &ReturnType) -> bool {
    match ty {
        ReturnType::Default => false,
        ReturnType::Type(_, ty) => match **ty {
            Type::Never(_) => true,
            _ => false,
        },
    }
}
