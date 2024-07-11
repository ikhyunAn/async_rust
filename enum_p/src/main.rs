use std::collections::{HashMap, HashSet};

// Define concrete types for `Job` and `CpuId`,
// otherwise you need to pass in Generics <Job, CpuId> to structs and functions where SchedulerState is used.
// Eliminates the need for generic paramters and makes code more rusty-like.
type Job = String;
type CpuId = u32;


pub enum SchedulerState {
    Inert,
    Pending(HashSet<Job>),
    Running(HashMap<CpuId, Vec<Job>>),
}


pub fn choose_scheduler(state: SchedulerState) {
    let output = match state {
        SchedulerState::Inert => "Inert",
        SchedulerState::Pending(hashset) => {
            println!("Size of the hashset is: {}", hashset.len());
            "Pending"
        }
        SchedulerState::Running(hashmap) => {
            let jobs = hashmap.get(&14).map_or_else(
                || "No jobs found".to_string(),
                |jobs| jobs.iter()
                            .map(|s| s.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")     //format!("{:?}", jobs)
            );
            println!("From hashmap with cpuid: {}, we got: {}", 14, jobs);
            "Running"
        }
    };
    println!("{output}");
}


fn main() {
    println!("Attempt 1");
    choose_scheduler(SchedulerState::Inert);
    
    println!("Attempt 2");
    let job1: Job = String::from("1st Job to do");
    let mut my_hashset: HashSet<Job> = HashSet::new();
    my_hashset.insert(job1);
    choose_scheduler(SchedulerState::Pending(my_hashset));
    
    println!("Attempt 3");
    let mycpu: CpuId = 14;
    let jobs1: Vec<Job> = vec!("job1".to_string(), String::from("job2"), String::from("job3"));
    let mut my_hashmap: HashMap<CpuId, Vec<Job>> = HashMap::new();
    my_hashmap.insert(mycpu, jobs1);

    choose_scheduler(SchedulerState::Running(my_hashmap));
}