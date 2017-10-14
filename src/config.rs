use std::fs::File;
use std::io::{Read, BufReader};

#[derive(Deserialize, Debug)]
pub struct ConfigV1 {
    pub test_image: Vec<TestImage>,
}

#[derive(Deserialize, Debug)]
pub struct TestImage {
    pub path: String,
    pub test_id: String,
}

pub fn read_config_file(path: &str) -> String {
    let input = File::open(path).expect("Could not find config file");
    let mut buffered = BufReader::new(input);

    let mut contents = String::new();
    buffered.read_to_string(&mut contents).expect("Could not read config file");

    contents
}
