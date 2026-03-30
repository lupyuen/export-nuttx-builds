#!/usr/bin/env bash
## Dump the GitHub PRs, Jobs and Job Durations for the past 2 days.
## Download and Parse all the GitHub Actions Logs.
## Export the Errors and Warnings into HTML and JSON.
## Push all updated data to GitHub for later analysis.

set -e  #  Exit when any command fails
set -x  #  Echo commands

## Loop forever
while true; do

  ## Dump the GitHub PRs, Jobs and Job Durations for the past 2 days.
  ## They will be populated in the pr, job and duration folders.
  ## Calls ../parse-nuttx-builds to download and parse the GitHub Build Logs.
  ## Which will populate the success, warning and error folders.
  pushd ../nuttx-github-jobs
  git pull
  ./dump-github-jobs.sh 2
  git pull && git add .
  git commit --all --message="Updated PRs, Jobs, Durations by \`dump-github-jobs.sh 2\`" && git push
  popd

  ## Join the PRs, Jobs and Durations into one TSV file and one JSON file:
  ## nuttx-github-jobs.tsv and nuttx-github-jobs.json
  pushd ../nuttx-github-jobs
  cargo run
  git pull && git add .
  git commit --all --message="Updated TSV and JSON by \`cargo run\`" && git push
  popd

  ## Export the PRs, Jobs, Durations, Build Logs into HTML and JSON
  pushd ../export-nuttx-builds
  git pull
  cargo run
  popd

  ## Push the Build Logs, HTML, JSON to GitHub for later analysis
  pushd ../nuttx-github-jobs
  git pull && git add .
  git commit --all --message="Updated Build Logs and HTML by \`export-nuttx-builds\`" && git push
  popd

  ## Wait a while so we don't hit GitHub API rate limits
  sleep 60

done
