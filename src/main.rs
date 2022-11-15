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
  let mut current_string_attribute_prefix: Option<String> = None;
  let mut current_string_attribute_name: Option<String> = None;
  let mut current_string_attribute_value: Option<String> = None;

  let mut current_float_attribute_name: Option<String> = None;
  let mut current_float_attribute_value: Option<f64> = None;

  let mut current_u64_attribute_name: Option<String> = None;
  let mut current_u64_attribute_value: Option<u64> = None;

  let mut in_extended_attribute = false;
  let mut in_key_value_pair = false;
  let mut in_extended_attr_key = false;
  let mut in_extended_attr_code_value = false;
  let mut current_extended_attr_key: Option<String> = None;
  let mut current_extended_attr_value: Option<String> = None;

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
        b"uro:extendedAttribute" => in_extended_attribute = true,
        b"uro:KeyValuePair" => in_key_value_pair = true,
        b"uro:key" => in_extended_attr_key = true,
        b"uro:codeValue" => in_extended_attr_code_value = true,
        b"gen:genericAttributeSet" => {
          let name_attr = e
            .attributes()
            .map(|x| x.unwrap())
            .find(|x| x.key.into_inner() == b"name")
            .unwrap();
          let name_attr_str = String::from_utf8_lossy(&name_attr.value).to_string();
          current_string_attribute_prefix = Some(format!("建築物::{}", name_attr_str));
        }
        b"gen:stringAttribute" => {
          let name_attr = e
            .attributes()
            .map(|x| x.unwrap())
            .find(|x| x.key.into_inner() == b"name")
            .unwrap();
          let name_attr_str = String::from_utf8_lossy(&name_attr.value).to_string();
          if let Some(prefix) = &current_string_attribute_prefix {
            current_string_attribute_name = Some(format!("{}::{}", prefix, name_attr_str));
          } else if name_attr_str == "建物ID" {
            current_string_attribute_name = Some("建築物::汎用属性::建物ID".to_string());
          } else {
            current_string_attribute_name = None;
          }
          current_string_attribute_value = None;
        }
        b"gen:measureAttribute" => {
          let name_attr = e
            .attributes()
            .map(|x| x.unwrap())
            .find(|x| x.key.into_inner() == b"name")
            .unwrap();
          let name_attr_str = String::from_utf8_lossy(&name_attr.value).to_string();
          if let Some(prefix) = &current_string_attribute_prefix {
            current_float_attribute_name = Some(format!("{}::{}", prefix, name_attr_str));
          } else {
            current_float_attribute_name = Some(name_attr_str);
          }
          current_float_attribute_value = None;
        }
        b"gml:name" => {
          current_string_attribute_name = Some("建築物::名称".to_string());
          in_value = true;
        }
        b"gen:value" => in_value = true,
        b"bldg:measuredHeight" => {
          current_float_attribute_name = Some("建築物::計測高さ".to_string());
          current_float_attribute_value = None;
          in_value = true;
        }
        b"xAL:LocalityName" => {
          current_string_attribute_name = Some("建築物::住所".to_string());
          in_value = true;
        }
        b"uro:buildingRoofEdgeArea" => {
          current_float_attribute_name = Some("建築物::建物利用現況::図上面積".to_string());
          current_float_attribute_value = None;
          in_value = true;
        }
        b"uro:districtsAndZonesType" => {
          current_u64_attribute_name = Some("建築物::建物利用現況::地域地区".to_string());
          current_u64_attribute_value = None;
          in_value = true;
        }
        b"uro:prefecture" => {
          current_u64_attribute_name = Some("建築物::建物利用現況::都道府県".to_string());
          current_u64_attribute_value = None;
          in_value = true;
        }
        b"uro:city" => {
          current_u64_attribute_name = Some("建築物::建物利用現況::市区町村".to_string());
          current_u64_attribute_value = None;
          in_value = true;
        }
        b"uro:surveyYear" => {
          current_u64_attribute_name = Some("建築物::建物利用現況::調査年".to_string());
          current_u64_attribute_value = None;
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
          current_string_attribute_value =
            Some(e.unescape().unwrap().to_string().trim().to_string());
        } else if current_float_attribute_name.is_some() && in_value {
          if let Ok(value) = e
            .unescape()
            .unwrap()
            .to_string()
            .trim()
            .to_string()
            .parse::<f64>()
          {
            current_float_attribute_value = Some(value);
          } else {
            current_float_attribute_value = None;
            // panic!(
            //   "Error parsing float attribute value: {:?}",
            //   e.unescape().unwrap().to_string()
            // );
          }
        } else if current_u64_attribute_name.is_some() && in_value {
          current_u64_attribute_value = Some(
            e.unescape()
              .unwrap()
              .to_string()
              .trim()
              .to_string()
              .parse::<u64>()
              .unwrap(),
          );
        } else if in_extended_attribute && in_key_value_pair && in_extended_attr_key {
          current_extended_attr_key = Some(e.unescape().unwrap().to_string().trim().to_string());
        } else if in_extended_attribute && in_key_value_pair && in_extended_attr_code_value {
          current_extended_attr_value = Some(e.unescape().unwrap().to_string().trim().to_string());
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
        b"uro:keyValuePair" => in_key_value_pair = false,
        b"uro:key" => in_extended_attr_key = false,
        b"uro:codeValue" => in_extended_attr_code_value = false,
        b"uro:extendedAttribute" => {
          // println!(
          //   "key: {:?}, value: {:?}",
          //   current_extended_attr_key, current_extended_attr_value
          // );
          in_extended_attribute = false;
          match current_extended_attr_key {
            Some(key) => {
              if key == "2" {
                let value = current_extended_attr_value.unwrap();
                current_properties.insert(
                  "建築物::拡張属性::LOD1の立ち上げに使用する建築物の高さ".to_string(),
                  serde_json::Value::String(value),
                );
              }
            }
            _ => (),
          }
          current_extended_attr_value = None;
          current_extended_attr_key = None;
        }
        b"gen:genericAttributeSet" => current_string_attribute_prefix = None,
        b"gen:stringAttribute" => {
          if let Some(name) = current_string_attribute_name {
            let value = current_string_attribute_value.unwrap();
            current_properties.insert(name, serde_json::Value::String(value));
          }
          current_string_attribute_name = None;
          current_string_attribute_value = None;
        }
        b"gen:measureAttribute" => {
          let name = current_float_attribute_name.unwrap();
          if let Some(value) = current_float_attribute_value {
            current_properties.insert(
              name,
              serde_json::Value::Number(serde_json::Number::from_f64(value).unwrap()),
            );
          } else {
            current_properties.insert(name, serde_json::Value::Null);
          }
          current_float_attribute_name = None;
          current_float_attribute_value = None;
        }
        b"gml:name" | b"xAL:LocalityName" => {
          in_value = false;
          let name = current_string_attribute_name.unwrap();
          let value = current_string_attribute_value.unwrap();
          current_properties.insert(name, serde_json::Value::String(value));
          current_string_attribute_name = None;
          current_string_attribute_value = None;
        }
        b"bldg:measuredHeight" | b"uro:buildingRoofEdgeArea" => {
          in_value = false;
          let name = current_float_attribute_name.unwrap();
          if let Some(value) = current_float_attribute_value {
            current_properties.insert(
              name,
              serde_json::Value::Number(serde_json::Number::from_f64(value).unwrap()),
            );
          } else {
            current_properties.insert(name, serde_json::Value::Null);
          }
          current_float_attribute_name = None;
          current_float_attribute_value = None;
        }
        b"uro:districtsAndZonesType" | b"uro:prefecture" | b"uro:city" | b"uro:surveyYear" => {
          in_value = false;
          let name = current_u64_attribute_name.unwrap();
          if let Some(value) = current_u64_attribute_value {
            current_properties.insert(
              name,
              serde_json::Value::Number(serde_json::Number::from(value)),
            );
          } else {
            current_properties.insert(name, serde_json::Value::Null);
          }
          current_u64_attribute_name = None;
          current_u64_attribute_value = None;
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
