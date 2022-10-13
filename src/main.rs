use clap::Parser;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[clap(
  name = "mbtiles_tool",
  about = "A tool for working with mbtiles archives",
  version
)]
struct Cli {
  #[clap(value_parser)]
  input: PathBuf,
}

struct FileToProcess {
  name: String,
  data: Vec<u8>,
}

struct Building {
  attributes: HashMap<String, String>,
}

fn process_one_file(file: FileToProcess) {
  println!("Processing file: {}", file.name);
  let mut xml_reader = quick_xml::Reader::from_reader(file.data.as_slice());
}

fn initialize_processors(
  process_rx: crossbeam::channel::Receiver<FileToProcess>,
) -> Vec<std::thread::JoinHandle<()>> {
  let mut threads = Vec::new();
  for _ in 0..num_cpus::get() {
    let process_rx = process_rx.clone();
    threads.push(std::thread::spawn(move || {
      for file in process_rx {
        process_one_file(file);
      }
    }));
  }
  threads
}

fn main() {
  let args = Cli::parse();
  let file = File::open(args.input).unwrap();
  let mut archive = zip::ZipArchive::new(file).unwrap();

  let (process_tx, process_rx) = crossbeam::channel::unbounded();
  let process_threads = initialize_processors(process_rx);

  for i in 0..archive.len() {
    let mut file = archive.by_index(i).unwrap();
    let parts = file.name().split('/').collect::<Vec<_>>();
    if parts.len() == 4 && parts[1] == "udx" && parts[2] == "bldg" && parts[3].ends_with(".gml") {
      // println!("{}", file.name());
      let mut buf = Vec::with_capacity(file.size() as usize);
      file.read_to_end(&mut buf).unwrap();
      process_tx
        .send(FileToProcess {
          name: file.name().to_string(),
          data: buf,
        })
        .unwrap();
    }
  }

  drop(process_tx);
  for thread in process_threads {
    thread.join().unwrap();
  }
}
