use quote::{ToTokens, quote};

fn main() {
    let bindings = bindgen::builder()
        .header_contents(
            "bindings.h",
            "
            #include <linux/input-event-codes.h>
        ",
        )
        .generate()
        .expect("Unable to generate bindings for input-event-codes.h");

    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());

    bindings
        .write_to_file(out_dir.join("input_bindings_raw.rs"))
        .unwrap();

    let parsed_bindings = syn::parse_str::<syn::File>(bindings.to_string().as_str())
        .expect("Failed to parse generated bindings");

    let mut enum_options = vec![];
    let mut int_to_keycode = vec![];
    let mut keycode_to_str = vec![];

    let mut used_keycode = std::collections::HashSet::<String>::new();

    for item in &parsed_bindings.items {
        let syn::Item::Const(const_var) = item else {
            continue;
        };
        if !const_var.ident.to_string().starts_with("KEY_") {
            continue;
        }

        let ident = &const_var.ident;
        let expr = &const_var.expr;

        let expr_str = expr.to_token_stream().to_string();
        if used_keycode.contains(&expr_str) {
            continue;
        }
        used_keycode.insert(expr_str);

        enum_options.push(quote! {
            #ident = #expr,
        });

        int_to_keycode.push(quote! {
            #expr => Ok(Self::#ident),
        });

        keycode_to_str.push(quote! {
            Self::#ident => stringify!(#ident),
        });
    }

    let module = quote! {
        #[allow(non_camel_case_types)]
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        pub enum Keycode {
            #(#enum_options)*
        }

        impl TryFrom<u16> for Keycode {
            type Error = ();

            fn try_from(val: u16) -> Result<Self, Self::Error> {
                match val {
                    #(#int_to_keycode)*
                    _ => Err(()),
                }
            }
        }

        impl std::fmt::Display for Keycode {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}",
                    match *self {
                        #(#keycode_to_str)*
                    }
                )
            }
        }
    };

    let keycode_ast = syn::parse2(module).unwrap();
    let formatted = prettyplease::unparse(&keycode_ast);

    std::fs::write(out_dir.join("keycode.rs"), formatted).unwrap();
}
