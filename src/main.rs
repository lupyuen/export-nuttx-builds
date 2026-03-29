use build_html::{Html, Table};
use struson::{
    json_path,
    reader::{JsonReader, JsonStreamReader, simple::{SimpleJsonReader, ValueReader}}, writer::{JsonStreamWriter, JsonWriter}
};

/// JSON File that contains the Job-PR records for all NuttX GitHub Jobs
const JOB_PR_JSON: &str = "../nuttx-github-jobs/nuttx-github-jobs.json";

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

    // Given a Run ID (Job ID): Locate the Job-PR
    let run_id = 23688473202;
    let json = 
        r#"
        [
            {"job_databaseId": 23688473202, "a": true, "b": true},
            {"job_databaseId": 23688473199, "a": false, "b": false}
        ]
        "#;
    let json_reader = SimpleJsonReader::new(json.as_bytes());
    // For each Job-PR record in the array...
    let mut index = Option::<usize>::None;
    let mut i = 0;
    json_reader.read_array_items(|array_reader| {
        // Fetch the Run ID: {"job_databaseId": 23688473202, ...
        array_reader.read_object_owned_names(|name, value_reader| {            
            // If the Run ID matches, remember the Found Index
            if name == "job_databaseId" {
                let val: u64 = value_reader.read_number().unwrap().unwrap();
                println!("{}: {}", name, val);
                if val == run_id {
                    index = Some(i);
                    println!("Found Index: {}", index.unwrap());
                }
            }
            Ok(())
        })?;
        println!("Item Done");
        i += 1;
        Ok(())
    }).unwrap();

    // Jump to the Found Index
    let index = index.unwrap() as u32;
    let mut json_reader = JsonStreamReader::new(json.as_bytes());
    let path = &json_path![index];
    // println!("json_reader before: {:?}\n", json_reader);
    json_reader.seek_to(path).unwrap();
    // println!("json_reader after: {:?}\n", json_reader);

    // Extract the Job-PR
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
