/// Implementation of Iguana workflow parsing

use serde::Deserialize;

use std::collections::HashMap;
use std::option::Option;

use linked_hash_map::LinkedHashMap;

mod job;

/// Container
#[derive(Debug, PartialEq, Deserialize)]
pub struct Container {
    image: String,
    env: Option<HashMap<String, String>>
}

/// Step
#[derive(Debug, PartialEq, Deserialize)]
pub struct Step {
    name: Option<String>,
    run: String,
    uses: Option<String>,
    with: Option<String>,
    env: Option<HashMap<String, String>>
}
/// Job
#[derive(Debug, PartialEq, Deserialize)]
pub struct Job {
    container: Container,
    services: Option<HashMap<String, Container>>,
    needs: Option<Vec<String>>,
    steps: Option<Vec<Step>>,
    #[serde(default)]
    continue_on_error: bool
}

/// Workflow
#[derive(Debug, PartialEq, Deserialize)]
pub struct Workflow {
    name: Option<String>,
    jobs: LinkedHashMap<String, Job>,
    env: Option<HashMap<String, String>>
}

pub fn do_workflow(workflow: String) -> Result<(), String> {
    let yaml_result: Result<Workflow, _> = serde_yaml::from_str(&workflow);

    let yaml = match yaml_result {
        Ok(r) => r,
        Err(e) => {
            return Err(format!("[ERROR] Unable to parse provided workflow file: {}", e));
        }
    };
 
    println!("Loaded control {}", yaml.name.unwrap_or("file".to_owned()));

    let jobs = yaml.jobs;

    if jobs.is_empty() {
        return Err("[ERROR] No jobs in control file!".to_owned());
    }

    let job_results = job::do_jobs(jobs, HashMap::new());

    match job_results {
        Ok(_) => println!("Workflow ran successfuly"),
        Err(e) => return Err(e)
    };
    Ok(())
}