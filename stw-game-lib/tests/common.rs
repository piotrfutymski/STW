use std::{fs, env};

use serde_json::{Value};


pub fn load_resources() -> Vec<(String, serde_json::Value)>{
    let mut res = vec![];

    for file in fs::read_dir(env::var("CARGO_MANIFEST_DIR").unwrap() + "/res").unwrap() {
        let data = fs::read_to_string(file.unwrap().path()).expect("Unable to read file");
    
        let json: Value = serde_json::from_str(&data).expect("Unable to parse json");
        let to_add: Vec<(String, serde_json::Value)> = json.as_array().expect("Json is not an array of resources")
            .into_iter()
            .map(|e| (
                    e.get("type").expect(&format!("element {} has no type", e)).as_str().expect(&format!("type in {} is not a string", e)).to_string(),
                    e.get("resource").expect(&format!("element {} has no resource", e)).clone()
                )
            ).collect();
        res.append(&mut to_add.clone());
    }
    res
    
}
