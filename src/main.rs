use std::fmt::format;
use std::fs::File;
use std::path::Path;
use std::thread;
use rand::Rng;
use serde::{Serialize};
use flate2::Compression;
use flate2::write::GzEncoder;
use tar::Header;
use clap::Parser;
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
use std::time::Duration;
use uuid::Uuid;

#[derive(Parser,Debug)]
struct Args {
    ///Paths to archive with testdata to create
    #[arg(short, long)]
    paths: Vec<String>,
    ///Count of testdata files in the archive
    #[arg(short, long)]
    count: u64,
    ///Threads per path
    #[arg(short, long)]
    threads_per_path: usize,
    ///Files per thread
    #[arg(short, long)]
    files_per_thread: usize
}

static GLOBAL_THREAD_COUNT: AtomicUsize = ATOMIC_USIZE_INIT;

fn main() -> std::io::Result<()>{
    let args:Args = Args::parse();

    for path in args.paths.clone(){
        for i in 0..args.threads_per_path{
            let path = path.clone();
            GLOBAL_THREAD_COUNT.fetch_add(1, Ordering::SeqCst);
            thread::spawn(move ||{
                for _ in 0..args.files_per_thread {
                    let tar_gz = File::create(gen_filepath(path.clone())).unwrap();
                    let enc = GzEncoder::new(tar_gz, Compression::best());
                    let mut archive = tar::Builder::new(enc);
                    for _ in 0..args.count {
                        let testdata = RandomTestData::new();
                        let json = serde_json::to_string(&testdata).unwrap();
                        //println!("{}",json);
                        let mut header = Header::new_gnu();
                        header.set_path(testdata.get_string_uid()).unwrap();
                        header.set_size(json.as_bytes().len() as u64);
                        header.set_cksum();
                        archive.append(&header, json.as_bytes()).unwrap();
                    }
                    archive.finish().unwrap();
                }
                GLOBAL_THREAD_COUNT.fetch_sub(1, Ordering::SeqCst);
            });

        }
    }
    // Wait for other threads to finish.
    while GLOBAL_THREAD_COUNT.load(Ordering::SeqCst) != 0 {
        thread::sleep(Duration::from_millis(1));
    }

    Ok(())
}

fn gen_filepath(path: String) -> String{
    let path = Path::new(path.as_str());
    path.join(Uuid::new_v4().to_string()).to_str().unwrap().to_string()
}

#[derive(Serialize)]
struct RandomTestData{
    ts: u64,
    user_id: String,
    data: String,
}

impl RandomTestData {

    pub fn new() -> Self{
        let mut rng = rand::thread_rng();
        Self{
            ts: rng.gen_range(15000000000..19000000000),
            user_id: rng.gen_range(0..100000000).to_string(),
            data: "testdata".to_string()
        }
    }
    pub fn get_string_uid(&self) -> String{
        format!("{}_{}.json",self.user_id,self.ts)
    }
}

