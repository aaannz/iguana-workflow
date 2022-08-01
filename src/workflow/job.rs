/// Implementation of job execution

use std::collections::HashMap;
use std::process::Command;

use linked_hash_map::LinkedHashMap;
use log::{error, warn, debug};

use crate::workflow::{Job, WorkflowOptions};

/// Available results of container run
#[derive(PartialEq)]
pub enum JobStatus {
    NoStatus,
    Skipped,
    Success,
    Failed
}

fn merge_from_ref(map: &mut HashMap<String, String>, map2: &HashMap<String, String>) {
    map.extend(map2.into_iter().map(|(k, v)| (k.clone(), v.clone())));
}

fn prepare_image(image: &String, dry_run: bool) -> Result<(), String> {
    let mut podman = Command::new("podman");
    let cmd = podman
        .args(["image", "pull", "--tls-verify=false", "--"])
        .arg(&image);

    debug!("{cmd:?}");

    if !dry_run {
        if let Err(e) = cmd.output() {
            return Err(e.to_string())
        }
    }
    Ok(())
}

fn run_container(name: &String, is_service: bool, env: HashMap<String, String>, dry_run: bool, debug: bool) -> Result<(), String> {
    let mut podman = Command::new("podman");
    let mut cmd = podman
        .args(["run", "--network=host", "--annotation=iguana=true", "--env=iguana=true"])
        .args(["--volume=/dev:/dev", "--mount=type=bind,source=/iguana,target=/iguana"]);

    if !is_service {
        cmd = cmd.args(["--tty", "--interactive"]);
    }
    else {
        cmd = cmd.arg("--detach");
    }

    if !debug {
        cmd = cmd.arg("--rm");
    }

    for (k, v) in env.iter() {
        cmd.arg(format!("--env={}={}", k, v));
    }

    cmd = cmd.args(["--", name]);

    debug!("{cmd:?}");

    if !dry_run {
        if let Err(e) = cmd.output() {
            return Err(e.to_string())
        }
    }
    Ok(())
}

fn do_job(name: &String, job: &Job, env_inherited: &Option<HashMap<String, String>>, opts: &WorkflowOptions) -> Result<(), String> {
    let image = &job.container.image;

    if image.len() == 0 {
        return Err(format!("No image specified for job {}", name))
    }
    debug!("Running job {}", name);
    let mut services_ok = true;
    // Prepare and run services
    match &job.services {
        Some(services) => {
            for (s_name, s_container) in services.iter() {
                match prepare_image(&s_container.image, opts.dry_run) {
                    Ok(()) => (),
                    Err(e) => {
                        error!("Preparation of service container '{}' failed: {}", s_name, e);
                        services_ok = false;
                        continue;
                    }
                }
                let mut env: HashMap<String, String> = HashMap::new();
                if env_inherited.is_some() {
                    merge_from_ref(&mut env, env_inherited.as_ref().unwrap());
                }
                if s_container.env.is_some() {
                    merge_from_ref(&mut env, s_container.env.as_ref().unwrap());
                }
                match run_container(&s_container.image, true, env, opts.dry_run, opts.debug) {
                    Ok(()) => debug!("Service '{}' started", s_name),
                    Err(e) => {
                        error!("Service container '{}' start failed: {}", s_name, e);
                        services_ok = false;
                    }
                }
            }
        }
        None => {}
    }

    if !services_ok {
        return Err(format!("Service container for job '{}' failed", name))
    }

    // Start main job
    match prepare_image(image, opts.dry_run) {
        Ok(()) => (),
        Err(e) => {
            return Err(format!("Preparation of container '{}' failed: {}", name, e))
        }
    }
    // Merge inherited and job specific environment
    let mut env: HashMap<String, String> = HashMap::new();
    if env_inherited.is_some() {
        let e = env_inherited.as_ref().unwrap();
        merge_from_ref(&mut env, e);
    }
    if job.container.env.is_some() {
        let e = job.container.env.as_ref().unwrap();
        merge_from_ref(&mut env, e);
    }
    match run_container(image, false, env, opts.dry_run, opts.debug) {
        Ok(()) => debug!("Job container '{}' started", image),
        Err(e) => {
            return Err(format!("Job container '{}' start failed: {}", image, e));
         }
    }

    Ok(())
}

/// Analyze "jobs" key of workflow and execute jobs in order
pub fn do_jobs(jobs: LinkedHashMap<String, Job>,
    mut jobs_status: HashMap<String, JobStatus>,
    env: &Option<HashMap<String, String>>,
    opts: &WorkflowOptions) -> Result<HashMap<String, JobStatus>, String> {
    // skip if job needs another one which already run and failed
    for (name, job) in jobs.iter() {
        jobs_status.insert(name.to_owned(), JobStatus::NoStatus);
        let mut skip = false;
        match &job.needs {
            Some(needs) => {
                for need in needs.iter() {
                    if ! jobs_status.contains_key(need) {
                        warn!("Job {} requires {} but this was not scheduled yet! Skipping check!", name, need);
                    }
                    else if jobs_status[need] == JobStatus::Failed {
                        warn!("Skipping job {} because of failed dependency {}", name, need);
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

        match do_job(name, job, env, opts) {
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