//! Export the Jobs, PRs and Builds from the NuttX GitHub Jobs into a Static HTML
use std::{thread::sleep, time::Duration};
use build_html::{Html, HtmlContainer, Table, TableCell, TableRow};
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
            sleep(Duration::from_secs(5));
            return;
        }
    };

    // Merge the Build JSON into the Job-PR JSON
    let merged_json = merge_build_json(build_json_path, &job_pr);
    let merged_json = match merged_json {
        Ok(json) => json,
        Err(e) => {
            println!("Error merging Build JSON: {e}");
            sleep(Duration::from_secs(5));
            return;
        }
    };
    println!("merged_json:\n{merged_json}\n");

    // Add the Merged JSON into a JSON Array
    let mut merged_json_array = Vec::<serde_json::Value>::new();
    let merged_json_value: serde_json::Value = serde_json::from_str(&merged_json).unwrap();
    merged_json_array.push(merged_json_value.clone());
    merged_json_array.push(merged_json_value.clone()); //// TODO

    // Sort the JSON Array by Timestamp in Descending Order (Latest First)
    merged_json_array.sort_by(|a, b| {
        let a_timestamp = a["build_timestamp"].as_str().unwrap_or_default();
        let b_timestamp = b["build_timestamp"].as_str().unwrap_or_default();
        b_timestamp.cmp(a_timestamp)
    });

    // Generate the HTML Table from Merged Job-PR-Build JSON
    let header = ["Timestamp", "PR", "Board / Config", "Error / Warning"];
    let mut table = Table::new()
        .with_attributes([("class", "table")])
        .with_header_row(header);
    for build_job_pr in merged_json_array {
        let timestamp = build_job_pr["build_timestamp"].as_str().unwrap_or_default();
        let pr = build_job_pr["pr_number"].as_u64().map(|n| n.to_string()).unwrap_or_default();
        let pr_url = build_job_pr["pr_url"].as_str().unwrap_or_default();
        let pr_title = build_job_pr["pr_title"].as_str().unwrap_or_default();
        let board = build_job_pr["build_board"].as_str().unwrap_or_default();
        let config = build_job_pr["build_config"].as_str().unwrap_or_default();
        let msg = build_job_pr["build_msg"].as_str().unwrap_or_default();
        let build_url = build_job_pr["build_url"].as_str().unwrap_or_default();

        let mut pr_title = pr_title.to_string();
        pr_title.truncate(50);

        let row = TableRow::new()
            .with_attributes([("class", "row")])
            .with_cell(TableCell::default()
                .with_attributes([("class", "timestamp")])
                .with_raw(timestamp)
            )
            .with_cell(TableCell::default()
                .with_attributes([("class", "pr")])
                .with_link(pr_url, format!("{pr}: {pr_title}"))
            )
            .with_cell(TableCell::default()
                .with_attributes([("class", "board-config")])
                .with_raw(format!("{board}:{config}"))
            )
            .with_cell(TableCell::default()
                .with_attributes([("class", "error-warning")])
                .with_link(build_url, msg.to_owned() + "<br><br>")
            );
        table.add_custom_body_row(row);
    }
    let html = table.to_html_string();
    println!("html:\n{html}");

    // Write the HTML Table to a Static File
    std::fs::write("/tmp/output.html", html).unwrap()
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
