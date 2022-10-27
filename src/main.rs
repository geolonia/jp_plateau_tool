use clap::Parser;
use geojson::{Feature, Geometry, Value};
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Parser)]
#[command(
  name = "jp_plateau_tool",
  about = "A tool to convert Plateau GML files to GeoJSON",
  version
)]
struct Cli {
  /// The input zip file containing gml files.
  #[arg()]
  input: PathBuf,

  /// The output GeoJSON file.
  #[arg(default_value = "./out.ndgeojson")]
  output: PathBuf,
}

struct FileToProcess {
  name: PathBuf,
  data: Vec<u8>,
}

fn poslist_to_coords(poslist: String) -> Vec<Vec<f64>> {
  let coords: Vec<f64> = poslist
    .split(' ')
    .map(|x| f64::from_str(x).unwrap())
    .collect();
  // this is a 3d polygon. strip the z values, and reverse the order of the x and y coordinates to match GeoJSON spec
  coords.chunks(3).map(|c| vec![c[1], c[0]]).collect()
}

fn process_one_file(file: &FileToProcess) -> Vec<Feature> {
  println!("Processing file: {}", file.name.display());
  let mut reader = quick_xml::Reader::from_reader(file.data.as_slice());
  let mut features: Vec<Feature> = Vec::new();

  let mut buf = Vec::new();
  let mut in_lod0_roof_edge = false;
  let mut in_poslist = false;

  let mut in_value = false;
  let mut current_string_attribute_name: Option<String> = None;
  let mut current_string_attribute_value: Option<String> = None;

  let mut current_float_attribute_name: Option<String> = None;
  let mut current_float_attribute_value: Option<f64> = None;

  let mut current_properties: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
  let mut current_poslist: Vec<Vec<Vec<f64>>> = vec![];
  loop {
    match reader.read_event_into(&mut buf) {
      Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
      Ok(quick_xml::events::Event::Eof) => break,
      Ok(quick_xml::events::Event::Start(e)) => match e.name().as_ref() {
        b"bldg:Building" => {
          current_properties = serde_json::Map::new();
        }
        b"bldg:lod0RoofEdge" => in_lod0_roof_edge = true,
        b"gml:posList" => {
          in_poslist = true;
        }
        b"gen:stringAttribute" => {
          let name_attr = e
            .attributes()
            .map(|x| x.unwrap())
            .find(|x| x.key.into_inner() == b"name")
            .unwrap();
          current_string_attribute_name =
            Some(String::from_utf8_lossy(&name_attr.value).to_string());
          current_string_attribute_value = None;
        }
        b"gml:name" => {
          current_string_attribute_name = Some("name".to_string());
          in_value = true;
        }
        b"gen:value" => in_value = true,
        b"bldg:measuredHeight" => {
          current_float_attribute_name = Some("measuredHeight".to_string());
          current_float_attribute_value = None;
          in_value = true;
        }
        _ => (),
      },
      Ok(quick_xml::events::Event::Text(e)) => {
        if in_lod0_roof_edge && in_poslist {
          let text = e.unescape().unwrap().to_string();
          let coords = poslist_to_coords(text);
          current_poslist.push(coords);
        } else if current_string_attribute_name.is_some() && in_value {
          current_string_attribute_value = Some(e.unescape().unwrap().to_string());
        } else if current_float_attribute_name.is_some() && in_value {
          current_float_attribute_value = Some(f64::from_str(&e.unescape().unwrap()).unwrap());
        }
      }
      Ok(quick_xml::events::Event::End(e)) => match e.name().as_ref() {
        b"bldg:Building" => {
          let poslist = std::mem::take(&mut current_poslist);

          let feature = Feature {
            bbox: None,
            geometry: Some(Geometry::new(Value::Polygon(poslist))),
            id: None,
            properties: Some(std::mem::replace(
              &mut current_properties,
              serde_json::Map::new(),
            )),
            foreign_members: None,
          };

          features.push(feature);
        }
        b"bldg:lod0RoofEdge" => in_lod0_roof_edge = false,
        b"gml:posList" => in_poslist = false,
        b"gen:value" => in_value = false,
        b"gml:name" => {
          in_value = false;
          let name = current_string_attribute_name.unwrap();
          let value = current_string_attribute_value.unwrap();
          current_properties.insert(name, serde_json::Value::String(value));
          current_string_attribute_name = None;
          current_string_attribute_value = None;
        }
        b"gen:stringAttribute" => {
          let name = current_string_attribute_name.unwrap();
          let value = current_string_attribute_value.unwrap();
          current_properties.insert(name, serde_json::Value::String(value));
          current_string_attribute_name = None;
          current_string_attribute_value = None;
        }
        b"bldg:measuredHeight" => {
          let name = current_float_attribute_name.unwrap();
          let value = current_float_attribute_value.unwrap();
          in_value = false;
          current_properties.insert(
            name,
            serde_json::Value::Number(serde_json::Number::from_f64(value).unwrap()),
          );
          current_float_attribute_name = None;
          current_float_attribute_value = None;
        }
        _ => (),
      },
      _ => (),
    }
    buf.clear();
  }

  features
}

fn initialize_processors(
  process_rx: crossbeam::channel::Receiver<FileToProcess>,
  output_tx: crossbeam::channel::Sender<Vec<u8>>,
) -> Vec<std::thread::JoinHandle<()>> {
  let thread_count = num_cpus::get();
  let mut threads = Vec::with_capacity(thread_count);
  for _ in 0..thread_count {
    let process_rx = process_rx.clone();
    let output_tx = output_tx.clone();
    threads.push(std::thread::spawn(move || {
      for file in process_rx {
        let features = process_one_file(&file);
        let mut buffer = Vec::new();
        for feature in features {
          let feature_json = serde_json::to_vec(&feature).unwrap();
          buffer.extend_from_slice(&feature_json);
          buffer.extend(b"\n");
        }
        output_tx.send(buffer).unwrap();
      }
    }));
  }
  threads
}

fn main() {
  let args = Cli::parse();
  crossbeam::scope(|s| {
    let mut out_file = File::options()
      .write(true)
      .create_new(true)
      .open(args.output)
      .unwrap();
    let file = File::open(args.input).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();

    let (process_tx, process_rx) = crossbeam::channel::unbounded();
    let (output_tx, output_rx) = crossbeam::channel::unbounded::<Vec<u8>>();
    let process_threads = initialize_processors(process_rx, output_tx);

    s.spawn(move |_| {
      for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let parts = file.name().split('/').collect::<Vec<_>>();
        if parts.len() == 4 && parts[1] == "udx" && parts[2] == "bldg" && parts[3].ends_with(".gml")
        {
          // println!("{}", file.name());
          let mut buf = Vec::with_capacity(file.size() as usize);
          file.read_to_end(&mut buf).unwrap();
          process_tx
            .send(FileToProcess {
              name: file.enclosed_name().unwrap().to_path_buf(),
              data: buf,
            })
            .unwrap();
        }
      }
    });

    s.spawn(move |_| {
      for buffer in output_rx {
        out_file.write_all(&buffer).unwrap();
      }
    });

    for thread in process_threads {
      thread.join().unwrap();
    }
  })
  .unwrap();
}
