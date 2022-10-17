use std::fs::File;
use std::io::{LineWriter, Write};
use std::path::Path;
use std::thread;
use rand::Rng;
use serde::{Serialize};
use flate2::Compression;
use flate2::write::{ZlibEncoder};
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
    files_per_thread: usize,
    ///Use raw blobs instead of tarballs
    #[arg(short, long,default_value_t = true)]
    blob: bool,
    ///GZ/Zlib compression, 2=best,1=fast,0=none
    #[arg(short, long, default_value_t=2)]
    gz_compression: u8
}

static GLOBAL_THREAD_COUNT: AtomicUsize = ATOMIC_USIZE_INIT;

fn main() -> std::io::Result<()>{
    let args:Args = Args::parse();

    for path in args.paths.clone(){
        for _ in 0..args.threads_per_path{
            let path = path.clone();
            GLOBAL_THREAD_COUNT.fetch_add(1, Ordering::SeqCst);
            thread::spawn(move ||{
                for _ in 0..args.files_per_thread {
                    if args.blob {
                        create_test_blob(path.clone(),args.count)
                    }else{
                        create_test_archive(path.clone(),args.gz_compression,args.count);
                    }
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

fn create_test_archive(path:String,gz_compression: u8,object_count: u64) {
    let file = File::create(gen_filepath(path)).unwrap();
    let compression: Compression;
    match gz_compression {
        0 => { compression = Compression::none() }
        1 => { compression = Compression::fast() }
        _ => { compression = Compression::best() }
    }
    let encoder = ZlibEncoder::new(file, compression);
    let mut archive = tar::Builder::new(encoder);
    for _ in 0..object_count {
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

fn create_test_blob(path:String,object_count:u64) {
    let file = File::create(gen_filepath(path)).unwrap();
    let mut file = LineWriter::new(file);
    for _ in 0..object_count{
        let testdata = RandomTestData::new();
        let json = serde_json::to_vec(&testdata).unwrap();
        file.write_all(json.as_slice()).unwrap();
        file.write_all(b"\n").unwrap();
    }
    file.flush().unwrap();
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

