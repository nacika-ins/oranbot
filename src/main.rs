extern crate mammut;
use mammut::Registration;
use mammut::apps::{AppBuilder, Scope};
use mammut::Mastodon;
use std::fs::File;
use std::io::prelude::*;
extern crate toml;
extern crate serde;
#[macro_use]
extern crate serde_derive;

mod bot;

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    global_string: Option<String>,
    global_integer: Option<u64>,
    app: Option<AppConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
struct AppConfig {
    client_id: Option<String>,
    client_secret: Option<String>,
    redirect: Option<String>,
    authorize_code: Option<String>,
    access_token: Option<String>
}

enum MODE{
    NONE,
    REGISTER,
    GET_AUTHORIZE_CODE,
    READY
}

fn main() {
    try().unwrap();
}

fn try() -> mammut::Result<()> {

    let mut app = get_app();
    let mut config = get_config();
    let mut registration = Registration::new("https://oransns.com")?;
    let mut mode = MODE::NONE;

    // Check if config is set, if not configure
    match config.app {
        Some(ref mut app_config) if app_config.client_id.is_none() => {
            println!("---> not found client_id");
            println!("---> generate client_id");
            registration.register(app)?;
            println!("---> get data for save");
            app_config.client_id = registration.client_id.clone();
            app_config.client_secret = registration.client_secret.clone();
            app_config.redirect = registration.redirect.clone();
            let url = registration.authorise()?;
            println!("---> please access to url: '{}'", url);
            println!("After accessing the URL, put the displayed authorize_code in config.toml");
            mode = MODE::REGISTER;
        }
        Some(ref mut app_config) if app_config.access_token.is_some() => {
            println!("---> foundaccess_token");
            mode = MODE::READY;
        }
        _ => {
            println!("--> found cliend_id");
            mode = MODE::GET_AUTHORIZE_CODE;
        }
    }

    match mode {

        // When the mode is changed to REGISTER, save processing is performed
        MODE::REGISTER => {
            save_config(&config);
        }
        MODE::GET_AUTHORIZE_CODE => {
            registration.client_id = config.app.as_ref().unwrap().client_id.clone();
            registration.client_secret = config.app.as_ref().unwrap().client_secret.clone();
            registration.redirect = config.app.as_ref().unwrap().redirect.clone();
            let authorize_code: String = match config.app.as_ref().unwrap().authorize_code.clone() {
                Some(ref authorize_code) if authorize_code == "" => { panic!("authorize_code is blank") }
                Some(authorize_code) => { authorize_code }
                None => { panic!("authorize_code is not included in config.toml") }
            };

            // Here you now need to open the url in the browser
            // And handle a the redirect url coming back with the code.
            println!("authorize_code ---> {}", authorize_code);
            let code = authorize_code;

            let mastodon: Mastodon = registration.create_access_token(code)?;
            let access_token = mastodon.data.token;
            match config.app {
                Some(ref mut app_config) => {
                    app_config.access_token = Some(access_token);
                }
                _ => { panic!() }
            }
            save_config(&config);
        }
        MODE::READY => {

            registration.client_id = config.app.as_ref().unwrap().client_id.clone();
            registration.client_secret = config.app.as_ref().unwrap().client_secret.clone();
            registration.redirect = config.app.as_ref().unwrap().redirect.clone();
            let access_token = config.app.as_ref().unwrap().access_token.clone().unwrap();
            let mastodon: Mastodon = registration.set_access_token(access_token)?;

            println!("--> ready");
            bot::exec(&mastodon);



        }
        _ => {}
    }

    Ok(())
}


// Get App
fn get_app<'a>() -> AppBuilder<'a> {
    AppBuilder {
        client_name: "oranbot",
        redirect_uris: "urn:ietf:wg:oauth:2.0:oob",
        scopes: Scope::All,
        website: None,
    }
}

// Save Config file
fn save_config(config: &Config) -> () {

    let t = toml::to_string(&*config).unwrap();
    let mut config_file: std::fs::File = File::create(&std::path::Path::new("config.toml")).unwrap();
    match config_file.write_all(t.as_bytes()) {
        Ok(_) => {
            println!("保存成功")
        }
        Err(e) => {
            println!("保存失敗 {:?}", e)
        }
    };
    drop(config_file);
}

// Open Config file
fn get_config() -> Config {

    let mut config_file: std::fs::File = match File::open(&std::path::Path::new("config.toml")) {
        Ok(config_file) => { println!("ファイルを開けました"); config_file }
        Err(e) => File::create(&std::path::Path::new("config.toml")).unwrap()
    };
    let mut config_text = String::new();
    config_file.read_to_string(&mut config_text);
    let mut config: Config = toml::from_str(&config_text).unwrap();
    if config.app.is_none() {
        config.app = Some(AppConfig {
            client_id: None,
            client_secret: None,
            authorize_code: Some("".to_owned()),
            access_token: None,
            redirect: None
        })
    }
    drop(config_file);
    config
}