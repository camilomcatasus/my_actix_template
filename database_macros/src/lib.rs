extern crate proc_macro;
use quote::{quote, ToTokens};
use proc_macro::TokenStream;
use syn::{ parse_macro_input, DeriveInput, Data, Fields, FieldsNamed, Ident};


#[proc_macro_derive(Queryable)]
pub fn print_tokens(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = parse_macro_input!(input as DeriveInput);
    let new_functions: proc_macro2::TokenStream;
    // Check if the input is a struct
    if let Data::Struct(data_struct) = ast.data {
        let struct_name = ast.ident;

        match data_struct.fields {
            Fields::Named(fields_named) => {
                let get_fn_tokens = body_get(&fields_named, &struct_name);
                let add_fn_tokens = body_add(&fields_named, &struct_name);
                let update_fn_tokens = body_update(&fields_named, &struct_name);
                new_functions = quote! {
                    impl #struct_name {
                        #get_fn_tokens
                        #add_fn_tokens
                        #update_fn_tokens
                    }
                }
            }
            _ => panic!("Only structs with named fields are supported"),
        }
    } else {
        panic!("Only structs are supported");
    }
    println!("{}", new_functions);
    return TokenStream::from(new_functions);
}

fn body_get(fields_named: &FieldsNamed, struct_name: &Ident) -> proc_macro2::TokenStream {

    let struct_name_string = String::from(struct_name.to_string());
    let idents: Vec<_> = fields_named.named.iter().map(|f| &f.ident).collect();
    let index: Vec<_> = fields_named.named.iter().enumerate().map(|f| f.0).collect();
    let types: Vec<_> = fields_named.named.iter().map(|f| &f.ty).collect();
    let conditions: Vec<String> = idents.iter().map(|ident| format!("AND {} = ", ident.as_ref().unwrap())).collect();
    quote! {
        pub fn get(conn: &rusqlite::Connection, #(#idents : Option<#types>),*) -> anyhow::Result<Self> {
            let mut query_string: String = format!("SELECT * FROM {} WHERE TRUE = TRUE", #struct_name_string);
            #(
                if let Some(i) = #idents {
                    query_string = format!("{}\n{}{}", query_string, #conditions, i);
                }
             )*
            let mut stmt = conn.prepare(&query_string)?;
            let obj = stmt.query_row([], |row| {
                Ok(#struct_name {
                    #(#idents : row.get(#index)?,)*
                })
            })?;

            return Ok(obj);
        }

        pub fn get_many(conn: &rusqlite::Connection, #(#idents : Option<#types>),*) -> anyhow::Result<Vec<Self>> {
            let mut query_string: String = format!("SELECT * FROM {} WHERE TRUE = TRUE", #struct_name_string);
            #(
                if let Some(i) = #idents {
                    query_string = format!("{}\n{}{}", query_string, #conditions, i);
                }
             )*

            let mut stmt = conn.prepare(&query_string)?;
            let obj_iter = stmt.query_map([], |row| {
                Ok(#struct_name {
                    #(#idents : row.get(#index)?,)*
                })
            })?;

            let obj_vector = obj_iter.map(|x| x.unwrap()).collect();

            return Ok(obj_vector);

        }
    }
}

fn body_add(fields_named: &FieldsNamed, struct_name: &Ident) -> proc_macro2::TokenStream {
    let struct_name_string = String::from(struct_name.to_string());
    let idents: Vec<_> = fields_named.named.iter().skip(1).map(|f| &f.ident).collect();
    let types: Vec<_> = fields_named.named.iter().skip(1).map(|f| &f.ty).collect();
    let type_strings: Vec<_> = types.into_iter().map(|t| {
        let mut token_stream = proc_macro2::TokenStream::new();
        t.to_tokens(&mut token_stream);
        let t_string: &str = &token_stream.to_string();
        match t_string {
            "String" => return String::from("\"{}\""),
            _ => return String::from("{}")
        }
    }).collect();
    let joined_brackets = type_strings.join(", ");
    
    let var_strings: Vec<_> = idents.iter().filter_map(|&opt| opt.as_ref()).map(|ident| ident.to_string()).collect();
    let joined_vars: String = var_strings.join(", ");
    let query_string: String = format!("INSERT INTO {} ({}) VALUES ({});", struct_name_string, joined_vars, joined_brackets);

    quote! {
        pub fn add(&self, conn: &rusqlite::Connection) -> anyhow::Result<usize> {
            let query_string: String = format!(#query_string #(, self.#idents)* );
            let stmt: usize = conn.execute(&query_string, ())?;  
            return Ok(stmt);
        }
    }
}

fn body_update(fields_named: &FieldsNamed, struct_name: &Ident) -> proc_macro2::TokenStream {
    let struct_name_string = String::from(struct_name.to_string());
    let idents: Vec<_> = fields_named.named.iter().map(|f| &f.ident).collect();
    let types: Vec<_> = fields_named.named.iter().map(|f| &f.ty).collect();
    let type_strings: Vec<_> = types.into_iter().map(|t| {
        let mut token_stream = proc_macro2::TokenStream::new();
        t.to_tokens(&mut token_stream);
        let t_string: &str = &token_stream.to_string();
        match t_string {
            "String" => return String::from("\"{}\""),
            _ => return String::from("{}")
        }
    }).collect();

    let first_ident = idents.get(0).unwrap();
    
    let var_strings: Vec<_> = idents.iter().filter_map(|&opt| opt.as_ref()).map(|ident| ident.to_string()).collect();
    let mut up_strings: Vec<String> = Vec::new();

    for index in 1..type_strings.len() {
        up_strings.push(format!("{} = {}", var_strings.get(index).unwrap(), type_strings.get(index).unwrap()));
    }

    let joined_up_strings: String = up_strings.join(",\n");
    let query_string: String = format!("UPDATE {} SET {} WHERE {} = {{}} ;", struct_name_string, joined_up_strings, var_strings.get(0).unwrap());
    let skipped_idents: Vec<_> = idents.iter().skip(1).map(|f| *f).collect();
    
    quote! {
        pub fn update(&self, conn: &rusqlite::Connection) -> anyhow::Result<usize> {
            let query_string: String = format!(#query_string #(, self.#skipped_idents)*, self.#first_ident);
            let stmt: usize = conn.execute(&query_string, ())?;  
            return Ok(stmt);
        }
    }
}
