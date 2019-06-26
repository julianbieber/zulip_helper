use failure::Error;
use std::fs::read_to_string;
use std::path::Path;

#[derive(Deserialize)]
pub struct Config {
    pub zulip_user: String,
    pub zulip_token: String,
    pub github_user: String,
    pub github_password: String,
    pub organisation: String,
}

lazy_static! {
    pub static ref CONFIG: Config = {
        let config_string = read(Path::new(".zulip_helper/config"));
        toml::from_str::<Config>(config_string.as_str()).expect("Config could not be parsed")
    };
}

pub static ZULIP_URL: &'static str = "https://zulip.patagona.de";

pub static GITHUB_URL: &'static str = "https://api.github.com";

fn read(path: &Path) -> String {
    dirs::home_dir()
        .map(|home| {
            let token_path = home.join(path);
            read_to_string(&token_path).expect(format!("{:?} does not exist", token_path).as_str())
        })
        .expect("home dir should be set")
}
