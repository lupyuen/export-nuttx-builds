//! Export the Jobs, PRs and Builds from the NuttX GitHub Jobs into a Static HTML
use std::{fs::read_dir, thread::sleep, time::Duration};
use build_html::{Html, HtmlContainer, Table, TableCell, TableCellType, TableRow};
use struson::{
    json_path,
    reader::{JsonReader, JsonStreamReader, simple::{SimpleJsonReader, ValueReader}},
    writer::{JsonStreamWriter, JsonWriter}
};

/// JSON File that contains the Job-PR records for all NuttX GitHub Jobs
const JOB_PR_JSON: &str = "../nuttx-github-jobs/nuttx-github-jobs.json";

fn main() {
    // Fetch the Recent Jobs from the Job-PR JSON
    let recent_jobs = fetch_recent_jobs();
    println!("Recent Jobs: {recent_jobs}\n");

    // Render the Recent Jobs as HTML Table
    let recent_jobs_html = render_recent_jobs(&recent_jobs);
    println!("Recent Jobs HTML:\n{recent_jobs_html}\n");

    // Remember the Merged Job-PR-Build JSON for each Run ID
    let mut merged_json_array = Vec::<serde_json::Value>::new();

    // Iterate through the Error and Warning Folders
    for folder in ["error", "warning"] {
        let path = format!("../nuttx-github-jobs/{folder}");
        if !std::path::Path::new(&path).exists() {
            println!("Folder {path} does not exist. Please parse-nuttx-builds first.");
            return;
        }

        // Iterate Backwards through all Run IDs (Job IDs) in the Error and Warning Folders
        // Like ../nuttx-github-jobs/error/23712816820
        let mut entries: Vec<_> = read_dir(&path).unwrap().collect();
        entries.sort_by_key(|entry| entry.as_ref().unwrap().path());
        for entry in entries.into_iter().rev() {
            let entry = entry.unwrap();
            let path = entry.path();
            println!("Found Build Path: {path:?}");

            // Run ID is the last part of the path: 23712816820
            let run_id = path.file_name().unwrap().to_str().unwrap();
            let run_id = run_id.parse::<u64>();
            let run_id = match run_id {
                Ok(id) => id,
                Err(e) => {
                    println!("Skipping invalid Run ID: {e}");
                    sleep(Duration::from_secs(1));
                    continue;
                }
            };
            println!("Run ID: {run_id}");

            // For each Run ID (Job ID), Fetch the Job-PR JSON
            let job_pr = fetch_job_pr(run_id);
            let job_pr = match job_pr {
                Ok(json) => json,
                Err(e) => {
                    println!("Error fetching Job-PR JSON: {e}");
                    sleep(Duration::from_secs(5));
                    continue;
                }
            };

            // Generate the Merged Job-PR-Build JSON for each Run ID:
            // Iterate through all Build JSON files in the folder
            // Like ../nuttx-github-jobs/error/23712816820/xtensa-03:lckfb-szpi-esp32s3:uvc.json
            let entries: Vec<_> = read_dir(&path).unwrap().collect();
            for entry in entries.into_iter() {
                let entry = entry.unwrap();
                let path = entry.path().to_str().unwrap().to_string();
                println!("Found Build JSON: {path}");

                // Merge the Build JSON into the Job-PR JSON
                let merged_json = merge_build_json(&path, &job_pr);
                let merged_json = match merged_json {
                    Ok(json) => json,
                    Err(e) => {
                        println!("Error merging Build JSON: {e}");
                        sleep(Duration::from_secs(5));
                        continue;
                    }
                };
                println!("merged_json:\n{merged_json}\n");

                // Add the Merged JSON into a JSON Array
                let merged_json_value: serde_json::Value = serde_json::from_str(&merged_json).unwrap();
                merged_json_array.push(merged_json_value.clone());

                // TODO: Stop iterating when Timestamp is Older than 5 Days
            }
        }
    }

    // Sort the JSON Array by Timestamp in Descending Order (Latest First)
    merged_json_array.sort_by(|a, b| {
        let a_timestamp = a["build_timestamp"].as_str().unwrap_or_default();
        let b_timestamp = b["build_timestamp"].as_str().unwrap_or_default();
        b_timestamp.cmp(a_timestamp)
    });

    // Write the JSON Array to a file
    let merged_json_array_str = serde_json::to_string_pretty(&merged_json_array).unwrap();
    std::fs::write("../nuttx-github-jobs/build-monitor.json", merged_json_array_str).unwrap();
    let recent_jobs_json_str = serde_json::to_string_pretty(&recent_jobs).unwrap();
    std::fs::write("../nuttx-github-jobs/build-monitor-pr.json", recent_jobs_json_str).unwrap();

    // Generate the HTML Table from Merged Job-PR-Build JSON
    let now = &chrono::Utc::now().to_rfc3339()[..19].replace("T", " ");
    let mut table = Table::new()
        .with_attributes([("class", "w-full text-left border-collapse whitespace-nowrap md:whitespace-normal")])
        .with_custom_header_row(
            TableRow::new()
                .with_attributes([("class", "bg-gray-50 border-b border-gray-200 text-xs uppercase tracking-wider text-gray-500 font-semibold")])
                .with_cell(TableCell::new(TableCellType::Header)
                    .with_attributes([("class", "px-6 py-4 w-32")])
                    .with_raw("Timestamp")
                )
                .with_cell(TableCell::new(TableCellType::Header)
                    .with_attributes([("class", "px-6 py-4 w-50")])
                    .with_raw("Pull Request")
                )
                .with_cell(TableCell::new(TableCellType::Header)
                    .with_attributes([("class", "px-6 py-4 min-w-[200px]")])
                    .with_raw("Board / Config")
                )
                .with_cell(TableCell::new(TableCellType::Header)
                    .with_attributes([("class", "px-6 py-4 min-w-[400px] w-full")])
                    .with_raw("Error / Warning")
                )
            )
        .with_tbody_attributes([("class", "divide-y divide-gray-100")]);

    // For every Merged Job-PR-Build...
    let mut prev_msg = None::<String>;
    for build_job_pr in merged_json_array {
        let timestamp = build_job_pr["build_timestamp"].as_str().unwrap_or_default();
        let pr = build_job_pr["pr_number"].as_u64().map(|n| n.to_string()).unwrap_or_default();
        let pr_url = build_job_pr["pr_url"].as_str().unwrap_or_default();
        let pr_title = build_job_pr["pr_title"].as_str().unwrap_or_default();
        let board = build_job_pr["build_board"].as_str().unwrap_or_default();
        let config = build_job_pr["build_config"].as_str().unwrap_or_default();
        let msg = build_job_pr["build_msg"].as_str().unwrap_or_default();
        let build_url = build_job_pr["build_url"].as_str().unwrap_or_default();
        let score = build_job_pr["build_score"].as_f64().unwrap_or_default();
        let mut pr_title = pr_title.to_string();
        pr_title.truncate(50);
        let timestamp = timestamp.replace("T", "<br>");

        // Shorten duplicate messages to "(Same)"
        let msg =
            if Some(msg.to_string()) == prev_msg {
                "(Same)".to_string()
            } else {
                prev_msg = Some(msg.to_string());
                msg.to_string()
            };

        // Render Errors in Red
        let error_warning = 
            if score == 0.0 { "bg-red-900" }
            else if score == 1.0 { "bg-green-900" }
            else { "bg-gray-900" };
        let error_warning = error_warning.to_string() + " px-6 py-4 block text-gray-300 rounded-lg p-3 font-mono text-xs leading-relaxed hover:bg-gray-800 transition-colors border border-gray-800 shadow-inner group-hover:border-gray-600 break-all whitespace-normal";

        let row = TableRow::new()
            .with_attributes([("class", "hover:bg-gray-50/80 transition-colors group align-top")])
            .with_cell(TableCell::default()
                .with_attributes([("class", "px-6 py-4 text-xs font-medium text-gray-900")])
                .with_raw(timestamp)
            )
            .with_cell(TableCell::default()
                .with_attributes([("class", "px-6 py-4 items-start gap-1.5 text-blue-600 hover:text-blue-800 hover:underline font-medium text-sm leading-snug break-words")])
                .with_link(pr_url, format!("{pr}: {pr_title}").replace(":", ":<br>"))
            )
            .with_cell(TableCell::default()
                .with_attributes([("class", "px-6 py-4 items-center px-2.5 py-1 rounded-md text-xs font-mono font-medium text-slate-800 border border-slate-200 break-all")])
                .with_raw(format!("{board}<br>:{config}"))
            )
            .with_cell(TableCell::default()
                .with_attributes([("class", error_warning.as_str())])
                .with_link(build_url, msg)
            );
        table.add_custom_body_row(row);
    }

    let header = format!
(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>NuttX Build Monitor</title>
    <!-- Import Tailwind CSS for styling -->
    <script src="https://cdn.tailwindcss.com"></script>
    <!-- Import Lucide Icons for some visual flair -->
    <script src="https://unpkg.com/lucide@latest"></script>
</head>
<body class="bg-gray-50 text-gray-800 p-4 md:p-8 font-sans antialiased">

    <div class="w-full mx-auto">

        <!-- Dashboard Header -->
        <div class="mb-6 flex flex-col md:flex-row md:items-center justify-between gap-4">
            <div>
                <h1 class="text-2xl font-bold text-gray-900 flex items-center gap-2">
                    <i data-lucide="activity" class="text-blue-600"></i>
                    NuttX Build Monitor
                </h1>
                <p class="text-sm text-gray-500 mt-1">Recent errors and warnings for NuttX GitHub CI</p>
            </div>
            <div class="text-sm text-gray-500 bg-white px-4 py-2 rounded-full border border-gray-200 shadow-sm flex items-center gap-2">
                <i data-lucide="clock" class="w-4 h-4"></i>
                Updated: {now} UTC
            </div>
        </div>

        <!-- Recent Jobs Table -->
        <div class="bg-white rounded-xl shadow-sm border border-gray-200 overflow-hidden">
            <!-- Responsive wrapper to prevent breaking on small screens -->
            <div class="overflow-x-auto">
                <!--
                {recent_jobs_html}
                -->
            </div>
        </div>

        <!-- Table Card -->
        <div class="bg-white rounded-xl shadow-sm border border-gray-200 overflow-hidden">
            <!-- Responsive wrapper to prevent breaking on small screens -->
            <div class="overflow-x-auto">
"#);

    let footer =
r#"
            </div>
        </div>
    </div>

    <!-- Initialize icons -->
    <script>
        lucide.createIcons();
    </script>
</body>
</html>
"#;
    let html = header.to_string() + &table.to_html_string() + footer;
    println!("html:\n{html}");

    // Write the HTML Table to a Static File
    std::fs::write("../nuttx-github-jobs/build-monitor.html", html).unwrap()
}

/// Scan the Job-PR JSON for Jobs that were started 24 hours ago or later.
/// Return the Jobs as a JSON Array.
/// Skip the earlier Jobs for the same PRs.
fn fetch_recent_jobs() -> serde_json::Value {
    // Open the Job-PR JSON File and create a Streaming JSON Reader
    let file = std::fs::read(JOB_PR_JSON).unwrap();
    let json_reader = SimpleJsonReader::new(file.as_slice());

    // For each Job-PR record in the array...
    let mut found_prs = Vec::<u64>::new();
    let mut recent_jobs = Vec::<u64>::new();
    json_reader.read_array_items(|array_reader| {
        // Fetch the Run ID, Started At and PR Number:
        // {"job_startedAt": "2026-04-01T22:06:23Z", "job_databaseId": 23873176516, "pr_number": 18654, ...
        let mut run_id = None::<u64>;
        let mut started_at = None::<String>;
        let mut pr_number = None::<u64>;
        array_reader.read_object_owned_names(|name, value_reader| {
            match name.as_str() {
                "job_databaseId" => {
                    let val: u64 = value_reader.read_number().unwrap().unwrap();
                    run_id = Some(val);
                },
                "pr_number" => {
                    let val: u64 = value_reader.read_number().unwrap().unwrap();
                    pr_number = Some(val);
                },
                "job_startedAt" => {
                    let val: String = value_reader.read_string().unwrap();
                    started_at = Some(val);
                },
                _ => {}
            }
            Ok(())
        })?;
        if run_id.is_none() || started_at.is_none() || pr_number.is_none() {
            return Err("Missing required fields".into());
        }
        let run_id = run_id.unwrap();
        let started_at = started_at.unwrap();
        let pr_number = pr_number.unwrap();

        // Stop if the Job-PR is Older than 24 Hours        
        let started_at = chrono::DateTime::parse_from_rfc3339(&started_at).unwrap();
        let now = chrono::Utc::now();
        if now.signed_duration_since(started_at) > chrono::Duration::hours(24) {
            return Err("Older than 24 hours".into());
        }

        // Skip if the PR was already found in an earlier Job-PR
        if found_prs.contains(&pr_number) { return Ok(()); }
        found_prs.push(pr_number);

        // Add the Job-PR to the Recent Jobs Array
        recent_jobs.push(run_id);
        Ok(())
    }).unwrap_or_default();

    // For each Recent Job-PR, Fetch the Job-PR JSON and add it to the Result Array
    let mut recent_jobs_json = Vec::<serde_json::Value>::new();
    for run_id in recent_jobs {
        let job_pr = fetch_job_pr(run_id);
        if let Ok(job_pr) = job_pr {
            let job_pr_value: serde_json::Value = serde_json::from_str(&job_pr).unwrap();
            recent_jobs_json.push(job_pr_value);
        }
    }
    serde_json::Value::Array(recent_jobs_json)
}

/// Render the Recent Jobs as HTML Table
fn render_recent_jobs(recent_jobs: &serde_json::Value) -> String {
    let mut html = String::new();
    for job_pr in recent_jobs.as_array().unwrap() {
        let run_id = job_pr["job_databaseId"].as_u64().unwrap_or_default();
        let pr_number = job_pr["pr_number"].as_u64().unwrap_or_default();
        let pr_url = job_pr["pr_url"].as_str().unwrap_or_default();
        let pr_title = job_pr["pr_title"].as_str().unwrap_or_default();
        let job_conclusion = job_pr["job_conclusion"].as_str().unwrap_or_default();
        let started_at = job_pr["job_startedAt"].as_str().unwrap_or_default();
        html += &format!("<tr><td><a href=\"{pr_url}\">PR #{pr_number}: {pr_title}</a> (Run ID: {run_id}) - {job_conclusion}</td></tr>\n");
    }
    format!("<table>{html}</table>")
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
    if let serde_json::Value::Object(ref mut job_pr_map) = job_pr_value
        && let serde_json::Value::Object(build_map) = build_value {
        for (key, value) in build_map {
            let key = format!("build_{key}")
                .replace("build_build_", "build_");
            job_pr_map.insert(key, value);
        }
    }
    let merged_json = serde_json::to_string_pretty(&job_pr_value)?;
    Ok(merged_json)
}
