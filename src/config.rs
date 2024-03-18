use const_format::concatcp;

pub const DATA_FOLDER: &str = "./_data";
pub const TEMP_SCRIPT_FOLDER: &str = concatcp!(DATA_FOLDER, "/temp/scripts");
pub const PROJECT_FOLDER: &str = concatcp!(DATA_FOLDER, "/projects");
pub const WEBHOOK_URL_PATH: &str = "/projects/webhook";
pub const STORE_LOCATION: &str = concatcp!(DATA_FOLDER, "/store/");
pub const STORE_FILE: &str = "store.json";
