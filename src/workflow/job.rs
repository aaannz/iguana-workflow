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

fn prepare_image(image: &String) -> Result<(), String> {
    let mut podman = Command::new("podman");
    let cmd = podman
        .args(["image", "pull", "--tls-verify=false", "--"])
        .arg(&image);

    if let Err(e) = cmd.output() {
        return Err(e.to_string())
    }
    Ok(())
}

fn run_container(name: &String, is_service: bool, env: &Option<HashMap<String, String>>) -> Result<(), String> {
    let mut podman = Command::new("podman");
    let mut cmd = podman
        .args(["run", "--rm", "--network=host", "--annotation=iguana=true", "--env=iguana=true"])
        .args(["--volume=/dev:/dev", "--mount=type=bind,source=/iguana,target=/iguana"]);

    if ! is_service {
        cmd = cmd.args(["--tty", "--interactive"]);
    }
    else {
        cmd = cmd.arg("--detach");
    }

    match env {
        Some(envs)=> {
            for (k, v) in envs.iter() {
                cmd.arg(format!("--env={}={}", k, v));
            }
        }
        None => {}
    }

    cmd = cmd.args(["--", name]);

    if let Err(e) = cmd.output() {
        return Err(e.to_string())
    }
    Ok(())
}

fn do_job(name: &String, job: &Job) -> Result<(), String> {
    let image = &job.container.image;

    if image.len() == 0 {
        return Err(format!("[ERROR] No image specified for job {}", name))
    }
    println!("[DEBUG] running job {}", name);
    let mut services_ok = true;
    // Prepare and run services
    match &job.services {
        Some(services) => {
            for (s_name, s_container) in services.iter() {
                match prepare_image(&s_container.image) {
                    Ok(()) => (),
                    Err(e) => {
                        println!("[ERROR] Preparation of service container '{}' failed: {}", s_name, e);
                        services_ok = false;
                        continue;
                    }
                }
                match run_container(&s_container.image, true, &s_container.env) {
                    Ok(()) => println!("[DEBUG] Service '{}' started", s_name),
                    Err(e) => {
                        println!("[ERROR] Service container '{}' start failed: {}", s_name, e);
                        services_ok = false;
                    }
                }
            }
        }
        None => {}
    }

    if !services_ok {
        return Err(format!("[ERROR] Service container for job '{}' failed", name))
    }

    // Start main job
    match prepare_image(image) {
        Ok(()) => (),
        Err(e) => {
            return Err(format!("[ERROR] Preparation of container '{}' failed: {}", name, e))
        }
    }
    match run_container(image, false, &job.container.env) {
        Ok(()) => println!("[DEBUG] Job container '{}' started", image),
        Err(e) => {
            return Err(format!("[ERROR] Job container '{}' start failed: {}", image, e));
         }
    }

    Ok(())
}