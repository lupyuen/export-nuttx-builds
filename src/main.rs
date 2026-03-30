//! Export the Jobs, PRs and Builds from the NuttX GitHub Jobs into a Static HTML
use std::thread::sleep;

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
    // Generate the Merged Job-PR-Build JSON for each Run ID
    // Stop iterating when Timestamp is Older than 5 Days
    // Sort by Timestamp in Descending Order (Latest First)

    // let run_id = 23653869993;  // sim-02:sim:login: >>>> WARNING: YOU ARE USING DEFAULT PASSWORD KEYS (CONFIG_FSUTILS_PASSWD_KEY1-4)!!! PLEASE CHANGE IT!!! <<<< \n 17d16 \n < CONFIG_BOARD_ETC_ROMFS_PASSWD_PASSWORD=\"Administrator\" \n Saving the new configuration file
    // let run_id = 23669957941;  // Successful
    // let run_id = 23679432579;  // Test Retry
    // let run_id = 1234;  // Doesn't exist
    let run_id = 23615674204; let build_json_path = "../nuttx-github-jobs/error/23615674204/xtensa-01:heltec_wifi_lora32:sx1276.json";  // Compile Error

    // For each Run ID (Job ID), Fetch the Job-PR JSON
    let job_pr = fetch_job_pr(run_id);
    let job_pr = match job_pr {
        Ok(json) => json,
        Err(e) => {
            println!("Error fetching Job-PR JSON: {e}");
            sleep(std::time::Duration::from_secs(5));
            return;
        }
    };
    println!("job_pr:\n{job_pr:?}\n");

    // Merge the Build JSON into the Job-PR JSON
    let merged_json = merge_build_json(build_json_path, &job_pr);
    let merged_json = match merged_json {
        Ok(json) => json,
        Err(e) => {
            println!("Error merging Build JSON: {e}");
            sleep(std::time::Duration::from_secs(5));
            return;
        }
    };
    println!("merged_json:\n{merged_json}\n");

    // TODO: Generate the HTML Table from Merged Job-PR-Build JSON:
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

/// Fetch the Job-PR JSON for a Given Run ID (Job ID)
fn fetch_job_pr(run_id: u64) -> Result<String, Box<dyn std::error::Error>> {
    // Open the Job-PR JSON File and create a Streaming JSON Reader
    let file = std::fs::read(JOB_PR_JSON)?;
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
                    // We simulate an Error to quit early
                    index = Some(i);
                    println!("Found Job-PR Index: {i}\n");
                    return Err(format!("{i}").to_string().into());
                }
            }
            Ok(())
        })?;
        i += 1;
        Ok(())
    }).unwrap_or_default();

    // Quit if index not found
    if index.is_none() {
        println!("Run ID {run_id} not found in {JOB_PR_JSON}. Please regenerate the JSON File.");
        return Err("Run ID not found".into());
    }
    let index = index.unwrap() as u32;

    // Jump to the Found Index in the Job-PR array
    let file = std::fs::read(JOB_PR_JSON)?;
    let mut json_reader = JsonStreamReader::new(file.as_slice());
    let path = &json_path![index];
    json_reader.seek_to(path)?;

    // Extract the Job-PR
    let mut writer = Vec::<u8>::new();
    let mut json_writer = JsonStreamWriter::new(&mut writer);
    json_reader.transfer_to(&mut json_writer)?;
    json_writer.finish_document()?;
    let job_pr = String::from_utf8(writer)?;

    // Validate the Job-PR JSON with Serde
    let job_pr2: serde_json::Value = serde_json::from_str(&job_pr)?;
    let job_pr3 = serde_json::to_string_pretty(&job_pr2)?;
    println!("job_pr:\n{job_pr3}\n");
    Ok(job_pr3)
}

/// Merge the Build JSON into the Job-PR JSON for a Given Run ID (Job ID)
fn merge_build_json(build_json_path: &str, job_pr: &str) -> Result<String, Box<dyn std::error::Error>> {
    let build_json = std::fs::read_to_string(build_json_path)?;
    let mut job_pr_value: serde_json::Value = serde_json::from_str(job_pr)?;
    let build_value: serde_json::Value = serde_json::from_str(&build_json)?;

    // Merge the Build JSON into the Job-PR JSON
    if let serde_json::Value::Object(ref mut job_pr_map) = job_pr_value {
        if let serde_json::Value::Object(build_map) = build_value {
            for (key, value) in build_map {
                let key = format!("build_{key}")
                    .replace("build_build_", "build_");
                job_pr_map.insert(key, value);
            }
        }
    }
    let merged_json = serde_json::to_string_pretty(&job_pr_value)?;
    Ok(merged_json)
}
