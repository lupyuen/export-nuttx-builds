//! Export the Jobs, PRs and Builds from the NuttX GitHub Jobs into a Static HTML
use build_html::{Html, Table};
use struson::{
    json_path,
    reader::{JsonReader, JsonStreamReader, simple::{SimpleJsonReader, ValueReader}},
    writer::{JsonStreamWriter, JsonWriter}
};

/// JSON File that contains the Job-PR records for all NuttX GitHub Jobs
const JOB_PR_JSON: &str = "../nuttx-github-jobs/nuttx-github-jobs.json";

fn main() {
    // Iterate Backwards through all Run IDs (Job IDs) in the Error and Warning Folders
    // Generate the Merged Build JSON for each Run ID
    // Stop iterating when Timestamp is Older than 5 Days
    // Sort by Timestamp in Descending Order (Latest First)
    fetch_job_pr();

    // TODO: Generate the HTML Table from Merged Build JSON:
    // Write the HTML Table to a Static File
    let header = ["Timestamp", "PR", "Error / Warning"];
    let source_table = [
        ["2026-04-01T12:00:02", "12345", "MCUBoot.zip unzip failed"],
        ["2026-04-01T12:00:01", "12346", "USE_LEGACY_PINMAP will be deprecated"],
        ["2026-04-01T12:00:00", "12347", "NIMBLE.zip unzip failed"]
    ];
    let html_table = Table::from(source_table)
        .with_header_row(header)
        .to_html_string();
    println!("HTML Table:\n{html_table}");
}

// Fetch the Job-PR-Build JSON for a Given Run ID (Job ID)
fn fetch_job_pr() {
    // Given a Run ID (Job ID): Locate the Job-PR from the JSON File
    // let run_id = 23653869993;  // sim-02:sim:login: >>>> WARNING: YOU ARE USING DEFAULT PASSWORD KEYS (CONFIG_FSUTILS_PASSWD_KEY1-4)!!! PLEASE CHANGE IT!!! <<<< \n 17d16 \n < CONFIG_BOARD_ETC_ROMFS_PASSWD_PASSWORD=\"Administrator\" \n Saving the new configuration file
    // let run_id = 23669957941;  // Successful
    let run_id = 23679432579;  // Test Retry
    // let run_id = 1234;  // Doesn't exist

    // Open the JSON File and create a Streaming JSON Reader
    let file = std::fs::read(JOB_PR_JSON).unwrap();
    let json_reader = SimpleJsonReader::new(file.as_slice());

    // For each Job-PR record in the array...
    let mut index = Option::<usize>::None;
    let mut i = 0;
    json_reader.read_array_items(|array_reader| {
        // Fetch the Run ID: {"job_databaseId": 23688473202, ...
        array_reader.read_object_owned_names(|name, value_reader| {            
            // If the Run ID matches, remember the Found Index
            if name == "job_databaseId" {
                let val: u64 = value_reader.read_number().unwrap().unwrap();
                if val == run_id {
                    index = Some(i);
                    println!("Found Job-PR Index: {i}\n");
                }
            }
            Ok(())
        })?;
        i += 1;
        Ok(())
    }).unwrap();

    // Quit if index not found
    if index.is_none() {
        println!("Run ID {run_id} not found in {JOB_PR_JSON}. Please regenerate the JSON File.");
        return;
    }
    let index = index.unwrap() as u32;

    // Jump to the Found Index in the Job-PR array
    let file = std::fs::read(JOB_PR_JSON).unwrap();
    let mut json_reader = JsonStreamReader::new(file.as_slice());
    let path = &json_path![index];
    json_reader.seek_to(path).unwrap();

    // Extract the Job-PR
    let mut writer = Vec::<u8>::new();
    let mut json_writer = JsonStreamWriter::new(&mut writer);
    json_reader.transfer_to(&mut json_writer).unwrap();
    json_writer.finish_document().unwrap() ;
    let job_pr = String::from_utf8(writer).unwrap();
    println!("job_pr:\n{job_pr}\n");

    // TODO: Merge the Build JSON into the Job-PR JSON    
}