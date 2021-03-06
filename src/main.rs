extern crate clap;
use clap::{Arg, App};

use std::path::PathBuf;
use std::io::Read;

extern crate reqwest;

#[macro_use]
extern crate serde_derive;
extern crate serde;

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate duct;

error_chain! {
    errors {
        InvalidStatus(status: reqwest::StatusCode) {
            description("invalid http status"),
            display("Invalid http status: status {}", status)
        }
    }

    foreign_links {
        ReqError(reqwest::Error);
        IoError(std::io::Error);
    }
}

#[derive(Deserialize, Debug)]
struct FileListData {
    id: u32,
    length: usize
}


fn make_search_request(search_term: &str, port: u32) -> Result<FileListData> {
    // Send a search request to the server
    let target_url = format!("http://localhost:{}/search?query={}", port, search_term);

    let mut response = reqwest::get(&target_url)?.error_for_status()?;

    if response.status() != reqwest::StatusCode::OK {
        bail!(ErrorKind::InvalidStatus(response.status()))
    }

    Ok(response.json()?)
}
fn make_list_info_request(list_id: u32, port: u32) -> Result<FileListData> {
    // Send a search request to the server
    let target_url = format!("http://localhost:{}/list?action=list_info&list_id={}", port, list_id);

    println!("target_url");

    let mut response = reqwest::get(&target_url)?.error_for_status()?;

    if response.status() != reqwest::StatusCode::OK {
        bail!(ErrorKind::InvalidStatus(response.status()))
    }

    Ok(response.json()?)
}
fn make_filename_requests(list: &FileListData, port: u32) -> Result<Vec<PathBuf>> {
    let mut result = vec!();

    for i in 0..list.length {
        let target_url = format!("http://localhost:{}/list?action=get_filename&list_id={}&index={}",
                                 port,
                                 list.id,
                                 i
                            );

        let mut response = reqwest::get(&target_url)?.error_for_status()?;

        if response.status() != reqwest::StatusCode::OK {
            bail!(ErrorKind::InvalidStatus(response.status()))
        }

        let mut buffer = String::new();
        response.read_to_string(&mut buffer);
        result.push(buffer);
    }

    let result = result.iter().map(|string| PathBuf::from(string)).collect();

    Ok(result)
}


fn main() {
    let matches = App::new("flash-cli")
        .version("0.1")
        .author("Frans Skarman")
        .about("Commandline interface for the flash image manager")
        .arg(Arg::with_name("search")
             .help("Search for a query and put the result of it in the target folder")
             .short("s")
             .long("search")
             .value_name("QUERY")
             .takes_value(true)
        )
        .arg(Arg::with_name("list")
             .help("create symlinks for all files in the list with the speicified id")
             .short("l")
             .long("list")
             .value_name("LIST_ID")
             .takes_value(true)
        )
        .arg(Arg::with_name("port")
             .required(true)
             .short("p")
             .long("port")
             .value_name("PORT")
             .help("Port to use when communicating with the server")
             .takes_value(true)
        )
        .arg(Arg::with_name("target_dir")
             .long("target_dir")
             .short("o")
             .value_name("TARGET_DIR")
             .takes_value(true)
             .help("Location to create symlinks to the files")
        )
        .arg(Arg::with_name("source_dir")
             .long("source_dir")
             .short("i")
             .value_name("SOURCE_DIR")
             .takes_value(true)
             .help("Directory where the files are stored")
        )
        .get_matches();

    let source_dir = PathBuf::from(
            matches.value_of("source_dir")
                .unwrap_or("")
        );
    let target_dir = PathBuf::from(
            matches.value_of("target_dir")
                .unwrap_or("/tmp/flash-cli")
        );
    let port = matches.value_of("port").and_then(|val| val.parse().ok()).unwrap();

    let list_data = if let Some(search_term) = matches.value_of("search") {
        make_search_request(search_term, port).unwrap()
    }
    else if let Some(list_id) = matches.value_of("list").and_then(|val| val.parse().ok()) {
        make_list_info_request(list_id, port).unwrap()
    }
    else {
        panic!("You must either specify a search term or a list id");
    };

    let files = make_filename_requests(&list_data, port).unwrap();

    for file in files {
        let full_target_path = target_dir.join(&file);
        let full_source_path = source_dir.join(&file);

        println!("{:?}", full_target_path);
        println!("{:?}", full_source_path);

        cmd!("ln", "-s", full_source_path, full_target_path).read().unwrap();
    }
}
