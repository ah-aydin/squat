extern crate proc_macro;

use std::collections::BTreeMap;

use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{self, DeriveInput, Type};

#[derive(deluxe::ExtractAttributes, Debug)]
#[deluxe(attributes(metadata))]
struct MetaDataDefinition {
    #[deluxe(default = String::from(""))]
    description: String,
}

#[derive(deluxe::ExtractAttributes, Debug)]
#[deluxe(attributes(arg))]
struct ArgDefinition {
    short: String,

    #[deluxe(default = String::from(""))]
    long: String,

    #[deluxe(default = String::from(""))]
    description: String,

    #[deluxe(default = false)]
    required: bool,
}

struct ArgData {
    short: String,
    long: Option<String>,
    description: Option<String>,
    required: bool,
    has_parameter: bool
}

impl ArgData {
    fn from_arg_attribs(attrs: ArgDefinition, has_parameter: bool) -> ArgData {
        ArgData {
            short: attrs.short,
            long: match attrs.long.len() {
                0 => None,
                _ => Some(attrs.long)
            },
            description: match attrs.description.len() {
                0 => None,
                _ => Some(attrs.description)
            },
            required: attrs.required,
            has_parameter 
        }
    }
}

fn is_bool(field: &Type) -> bool {
    if let Type::Path(type_path) = field {
        if let Some(path_segment) = type_path.path.segments.first() {
            let field_type = path_segment.ident.to_string();
            return field_type == "bool";
        }
        panic!("WTF IS THIS I DUNNO 2");
    }
    panic!("WTF IS THIS I DUNNO");
}

fn extract_arg_field_attrs(ast: &mut DeriveInput) -> deluxe::Result<BTreeMap<String, ArgData>> {
    let mut field_attrs: BTreeMap<String, ArgData> = BTreeMap::new();

    if let syn::Data::Struct(s) = &mut ast.data {
        for field in s.fields.iter_mut() {
            let field_name = field.ident.as_ref().unwrap().to_string();
            let attrs: ArgDefinition = deluxe::extract_attributes(field)?;
            field_attrs.insert(field_name, ArgData::from_arg_attribs(attrs, !is_bool(&field.ty)));
        }
    } else {
        panic!("Only structs are supported");
    }

    Ok(field_attrs)
}

fn build_match_arms(index: usize, value: &str, field_name: &String, has_parameter: bool) -> proc_macro2::TokenStream {
    let value_str = syn::LitStr::new(value, proc_macro2::Span::call_site());
    let field_name = format_ident!("{}", field_name);
    let set_struct_field = match has_parameter {
        true => quote! {
            i += 1;
            let arg = args[i].as_str();
            if arg.starts_with("-") {
                error(&format!("'{}' requires an argument to be passed", #value_str));
            }

            return_struct.#field_name = arg.to_owned();
        },
        false => quote! {
            return_struct.#field_name = true;
        }
    };
    quote! {
        #value_str => {
            if !processed[#index] {
                processed[#index] = true;
            } else {
                error(&format!("Option '{}' has allready been processed", #value_str));
            }

            #set_struct_field
        }
    }
}

fn parse_cmd_args_derive2(item: proc_macro2::TokenStream) -> deluxe::Result<proc_macro2::TokenStream> {
    // parse
    let mut ast: DeriveInput = syn::parse2(item)?;
    let meta_data: MetaDataDefinition = deluxe::extract_attributes(&mut ast)?;

    // extract field attributes
    let field_attrs: BTreeMap<String, ArgData> = extract_arg_field_attrs(&mut ast)?;

    // Destructure the map and build debug strings
    let mut field_names = Vec::new();
    let mut short_commands= Vec::new();
    let mut long_commands = Vec::new();
    let mut required = Vec::new();
    let mut has_parameter = Vec::new();

    let mut usage_str = String::from("Usage: EXEC_NAME [OPTIONS]");
    let mut options_str = String::from("");

    for (field, attr) in field_attrs {
        let short = attr.short.clone();
        let long;
        let req = attr.required;
        let description;

        field_names.push(field.clone());

        short_commands.push(short.clone());

        required.push(req);
        has_parameter.push(attr.has_parameter);

        if let Some(val) = attr.long {
            long = val.clone();
            long_commands.push(val.clone());
        } else {
            long = "".to_owned();
            long_commands.push("".to_owned());
        }

        if let Some(val) = attr.description {
            description = val;
        } else {
            description = "".to_owned();
        }

        if req {
            usage_str += &format!(" {} {}", short, field);
        }
        let parameter;
        if attr.has_parameter {
            parameter = format!("<{}>", field.to_uppercase());
        } else {
            parameter = "".to_owned();
        }
        options_str += &format!("   {:03} {:20} {:20} {}", short, long, parameter, description);
        if req {
            options_str += " (Required)";
        }
        options_str += "\n";
    }
    options_str += &format!("   {:03} {:20} {:20} {}", "-h", "--help", "", "Displays help");

    if meta_data.description.len() > 0 {
        usage_str += &format!("\n\n{}", &meta_data.description);
    }

    let options_count = field_names.len();

    let ident = &ast.ident;
    let (impl_generics, type_generics, where_clause) = ast.generics.split_for_impl();

    // Generate 
    let match_arms_short_commands = short_commands.iter().enumerate().map(|(index, value)| {
        build_match_arms(index, value, &field_names[index], has_parameter[index])
    });
    let match_arms_long_commands = long_commands.iter().enumerate().map(|(index, value)| {
        build_match_arms(index, value, &field_names[index], has_parameter[index])
    });

    let code = quote! {
        impl #impl_generics #ident #type_generics #where_clause {
            pub fn parse() -> #ident {
                fn error(msg: &str) {
                    println!("{}", msg);
                    print_help();
                }

                fn print_help() {
                    let mut usage_str = String::from(#usage_str);
                    usage_str = usage_str
                        .replace(
                            "EXEC_NAME",
                            std::env::current_exe()
                                .ok()
                                .unwrap()
                                .file_name()
                                .unwrap()
                                .to_str()
                                .unwrap()
                        );
                    println!("{}", usage_str);
                    println!("\nOptions:");
                    println!("{}", #options_str);
                    std::process::exit(1);
                }

                let mut return_struct: #ident = Default::default();
                let mut processed = vec![false; #options_count];
                let mut required = vec![#(#required),*];
                let mut args: Vec<String> = std::env::args().collect();

                args.remove(0);
                let mut i = 0;
                while i < args.len() {
                    let arg = args[i].as_str();
                    match arg {
                        #(#match_arms_short_commands),*
                        ,
                        #(#match_arms_long_commands),*
                        ,
                        "-h" => {
                            print_help();
                        },
                        "--help" => {
                            print_help();
                        },
                        _ => {
                            error(&format!("[ERROR] '{}' is not a valid option", arg));
                        }
                    };
                    i += 1;
                }
                
                for i in 0..#options_count {
                    if required[i] && !processed[i] {
                        error(&format!("[ERROR] not all required options have been provided."));
                    }
                }

                return_struct
            }
        }
    };

    Ok(code)
}

#[proc_macro_derive(CmdArgs, attributes(metadata, arg))]
pub fn parse_cmd_args_derive(input: TokenStream) -> TokenStream {
    parse_cmd_args_derive2(input.into()).unwrap().into()
}
