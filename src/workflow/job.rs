/// Implementation of job execution

use std::collections::HashMap;
use std::process::Command;

use linked_hash_map::LinkedHashMap;

use crate::workflow::{Job, Container, Step};

#[derive(PartialEq)]
pub enum JobStatus {
    NoStatus,
    Skipped,
    Success,
    Failed
}

/// Analyze "jobs" key of workflow and execute jobs in order
pub fn do_jobs(jobs: LinkedHashMap<String, Job>, mut jobs_status: HashMap<String, JobStatus>) -> Result<HashMap<String, JobStatus>, String> {
    // skip if job needs another one which already run and failed
    for (name, job) in jobs.iter() {
        jobs_status.insert(name.to_owned(), JobStatus::NoStatus);
        let mut skip = false;
        match &job.needs {
            Some(needs) => {
                for need in needs.iter() {
                    if ! jobs_status.contains_key(need) {
                        println!("[WARNING] Job {} requires {} but this was not scheduled yet! Skipping check!", name, need);
                    }
                    else if jobs_status[need] == JobStatus::Failed {
                        println!("[WARNING] Skipping job {} because of failed dependency {}", name, need);
                        skip = true;
                        break;
                    }
                }
            }
            None => {}
        }
        if skip {
            jobs_status.insert(name.to_owned(), JobStatus::Skipped);
            continue;
        }
        
        match do_job(name, job) {
            Ok(()) => {
                jobs_status.insert(name.to_owned(), JobStatus::Success);
            }
            Err(e) => {
                jobs_status.insert(name.to_owned(), JobStatus::Failed);
                if !job.continue_on_error {
                    return Err(e);
                }
            }
        }
    }
    Ok(jobs_status)
}

fn do_job(name: &String, job: &Job) -> Result<(), String> {
    let image = &job.container;

    if image.len() == 0 {
        return Err(format!("[ERROR] No image specified for job {}", name))
    }
    println!("[DEBUG] running job {}", name);
    Ok(())
}