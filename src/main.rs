use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{fs, fs::File};

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
  name: PathBuf,
  data: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Geometry {
  #[serde(rename = "type")]
  type_: String,
  coordinates: Vec<Vec<Vec<f64>>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Feature {
  #[serde(rename = "type")]
  type_: String,
  geometry: Geometry,
  properties: serde_json::Map<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug)]
struct FeatureCollection {
  #[serde(rename = "type")]
  type_: String,
  features: Vec<Feature>,
}

fn poslist_to_geometry(poslist: String) -> Geometry {
  let coords: Vec<f64> = poslist
    .split(' ')
    .map(|x| f64::from_str(x).unwrap())
    .collect();
  // this is a 3d polygon. strip the z values, and reverse the order of the x and y coordinates to match GeoJSON spec
  let coords = coords.chunks(3).map(|c| vec![c[1], c[0]]).collect();

  Geometry {
    type_: "Polygon".to_string(),
    coordinates: vec![coords],
  }
}

fn process_one_file(file: FileToProcess) {
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
  let mut current_geometry: Geometry = Geometry {
    type_: "Polygon".to_string(),
    coordinates: vec![],
  };
  loop {
    match reader.read_event_into(&mut buf) {
      Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
      Ok(quick_xml::events::Event::Eof) => break,
      Ok(quick_xml::events::Event::Start(e)) => match e.name().as_ref() {
        b"bldg:Building" => {
          current_properties = serde_json::Map::new();
          // let attrs = e.attributes().map(|attr| attr.unwrap()).collect::<Vec<_>>();
          // let gml_id = attrs
          //   .iter()
          //   .find(|attr| -> bool { attr.key.into_inner() == b"gml:id" })
          //   .unwrap()
          //   .unescape_value()
          //   .unwrap();
          // current_properties.insert("gml_id".to_string(), gml_id.to_string());
          // println!("Found a building: {}", gml_id);
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
        b"gen:value" => in_value = true,
        b"bldg:measuredHeight" => {
          current_float_attribute_name = Some("measuredHeight".to_string());
          current_float_attribute_value = None;
          in_value = true;
        }
        _ => (),
        // other => println!(
        //   "Found something else: {}",
        //   std::str::from_utf8(other).unwrap()
        // ),
      },
      Ok(quick_xml::events::Event::Text(e)) => {
        if in_lod0_roof_edge && in_poslist {
          let text = e.unescape().unwrap().to_string();
          let geom = poslist_to_geometry(text);
          // println!(
          //   "roof edge poslist: {}",
          //   serde_json::to_string(&geom).unwrap()
          // );
          current_geometry = geom;
        } else if let Some(_) = &current_string_attribute_name {
          if in_value {
            current_string_attribute_value = Some(e.unescape().unwrap().to_string());
          }
        } else if let Some(_) = &current_float_attribute_name {
          if in_value {
            current_float_attribute_value = Some(f64::from_str(&e.unescape().unwrap()).unwrap());
          }
        }
      }
      Ok(quick_xml::events::Event::End(e)) => match e.name().as_ref() {
        b"bldg:Building" => {
          let feature = Feature {
            type_: "Feature".to_string(),
            geometry: std::mem::replace(
              &mut current_geometry,
              Geometry {
                type_: "Polygon".to_string(),
                coordinates: vec![],
              },
            ),
            properties: std::mem::replace(&mut current_properties, serde_json::Map::new()),
          };
          features.push(feature);
        }
        b"bldg:lod0RoofEdge" => in_lod0_roof_edge = false,
        b"gml:posList" => in_poslist = false,
        b"gen:value" => in_value = false,
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
        // other => println!(
        //   "Found the end of something else: {}",
        //   std::str::from_utf8(other).unwrap()
        // ),
      },
      _ => (),
    }
    buf.clear();
  }

  // println!("{}", serde_json::to_string(&out_collection).unwrap());
  let out_file_name = Path::new("./out/").join(format!(
    "{}.ndgeojson",
    file.name.file_stem().unwrap().to_string_lossy()
  ));
  let mut out_file = File::create(out_file_name).unwrap();
  for feature in features {
    let feature_json = serde_json::to_string(&feature).unwrap();
    out_file.write_all(feature_json.as_bytes()).unwrap();
    out_file.write_all(b"\n").unwrap();
  }
}

fn initialize_processors(
  process_rx: crossbeam::channel::Receiver<FileToProcess>,
) -> Vec<std::thread::JoinHandle<()>> {
  let thread_count = num_cpus::get();
  let mut threads = Vec::with_capacity(thread_count);
  for _ in 0..thread_count {
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

  fs::create_dir("./out").unwrap();

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
          name: file.enclosed_name().unwrap().to_path_buf(),
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
