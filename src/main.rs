use std::thread::sleep;

use build_html::{Html, Table};
use struson::{
    json_path,
    reader::{
        JsonStreamReader, JsonSyntaxError, ReaderError, SyntaxErrorKind, ValueType,
        simple::{SimpleJsonReader, ValueReader},
    },
};

fn main() {
    println!("export-nuttx-builds");
    let source_table = [
        [1, 2, 3],
        [4, 5, 6],
        [7, 8, 9]
    ];
    let html_table = Table::from(source_table)
        .with_header_row(['A', 'B', 'C'])
        .to_html_string();
    println!("{}", html_table);

    let json = 
        r#"
        [
            {"nested1": true},
            {"nested2": true}
        ]
        "#;
    let json_reader = SimpleJsonReader::new(json.as_bytes());
    let return_value = json_reader.read_array_items(|mut array_reader| {
        array_reader.read_object_owned_names(|name, value_reader| {            
            let val = value_reader.read_bool().unwrap();
            println!("{}: {}", name, val);
            Ok(())
        })?;
        Ok(())
    }).unwrap();
    println!("return_value: {:?}", return_value);
}
