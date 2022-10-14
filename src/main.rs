use std::fs::File;
use rand::Rng;
use serde::{Serialize};
use flate2::Compression;
use flate2::write::GzEncoder;
use tar::Header;
use clap::Parser;

#[derive(Parser,Debug)]
struct Args {
    ///Path to archive with testdata to create
    #[arg(short, long)]
    path: String,
    ///Count of testdata files in the archive
    #[arg(short, long, default_value_t = 1000)]
    count: u64,
}

fn main() -> std::io::Result<()>{
    let args:Args = Args::parse();

    let tar_gz = File::create(args.path)?;
    let enc = GzEncoder::new(tar_gz,Compression::best());
    let mut archive = tar::Builder::new(enc);
    for _ in 0..args.count {
        let testdata = RandomTestData::new();
        let json = serde_json::to_string(&testdata)?;
        //println!("{}",json);
        let mut header = Header::new_gnu();
        header.set_path(testdata.get_string_uid())?;
        header.set_size(json.as_bytes().len() as u64);
        header.set_cksum();
        archive.append(&header,json.as_bytes())?;
    }
    archive.finish()?;
    Ok(())
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

