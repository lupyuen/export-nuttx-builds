use build_html::{Html, Table};
use struson::{
    reader::{
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

    // Given a Run ID (Job ID): Locate the PR
    let run_id = 1234;
    let json = 
        r#"
        [
            {"a1": true, "a2": false},
            {"b1": true, "b2": false}
        ]
        "#;
    let json_reader = SimpleJsonReader::new(json.as_bytes());
    let return_value = json_reader.read_array_items(|array_reader| {
        // If the Run ID matches, remember the PR
        array_reader.read_object_owned_names(|name, value_reader| {            
            let val = value_reader.read_bool().unwrap();
            println!("{}: {}", name, val);
            Ok(())
        })?;
        println!("Item Done");
        Ok(())
    }).unwrap();
    println!("return_value: {:?}", return_value);
}
