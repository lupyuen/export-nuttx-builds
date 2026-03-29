use build_html::{Html, Table};
use struson::{
    json_path,
    reader::{JsonReader, JsonStreamReader, simple::{SimpleJsonReader, ValueReader}}, writer::{JsonStreamWriter, JsonWriter}
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

    // Given a Run ID (Job ID): Locate the Job + PR
    let run_id = 1234;
    let json = 
        r#"
        [
            {"a": true, "b": true},
            {"a": false, "b": false}
        ]
        "#;
    let json_reader = SimpleJsonReader::new(json.as_bytes());
    let return_value = json_reader.read_array_items(|array_reader| {
        // If the Run ID matches, remember the Job + PR
        // println!("array_reader: {:?}", array_reader);
        array_reader.read_object_owned_names(|name, value_reader| {            
            let val = value_reader.read_bool().unwrap();
            println!("{}: {}", name, val);
            Ok(())
        })?;
        println!("Item Done");
        Ok(())
    }).unwrap();
    println!("return_value: {:?}", return_value);

    // Jump to the Found Index and read the Job + PR
    let index = 1;
    let mut json_reader = JsonStreamReader::new(json.as_bytes());
    let path = &json_path![index];
    // println!("json_reader before: {:?}\n", json_reader);
    json_reader.seek_to(path).unwrap();
    // println!("json_reader after: {:?}\n", json_reader);

    // Extract the Job + PR
    let mut writer = Vec::<u8>::new();
    let mut json_writer = JsonStreamWriter::new(&mut writer);
    // json_writer.begin_object().unwrap();
    // json_writer.name("embedded").unwrap();
    json_reader.transfer_to(&mut json_writer).unwrap();
    // json_writer.end_object().unwrap();
    json_writer.finish_document().unwrap() ;
    let job_pr = String::from_utf8(writer).unwrap();
    println!("job_pr: {}", job_pr);
}
