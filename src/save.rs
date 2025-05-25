use ron::{de::from_str, ser::to_string};
use std::fs;
use crate::editor::GameObject;

pub fn save_scene(objects: &Vec<GameObject>, path: &str) {
    if let Ok(ron_string) = to_string(objects) {
        let _ = fs::write(path, ron_string);
    }
}

pub fn load_scene(path: &str) -> Vec<GameObject> {
    if let Ok(content) = fs::read_to_string(path) {
        if let Ok(objs) = from_str::<Vec<GameObject>>(&content) {
            return objs;
        }
    }
    Vec::new()
}