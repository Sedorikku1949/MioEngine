//use crate::utils;
//use rust_i18n::t;
//
//rust_i18n::i18n!("locales");
//
//
//pub fn load(logs: bool){
//  //i18n!("locales");
//  rust_i18n::set_locale("fr-FR");
//  if logs {
//    utils::success("i18n", "Language modules ready to use")
//  }
//}
//
//pub fn test(){
//  dbg!(&t!("hello", locale = "fr"));
//  dbg!(&t!("hello", locale = "en"));
//}

use crate::utils;
use std::{ collections::HashMap, path::Path, fs, process::exit };
use once_cell::sync::Lazy;
use serde_json::Value;
use serenity::prelude::RwLock;

//static GLOBAL_DATA: Lazy<RwLock<HashMap<i32, String>>> = Lazy::new(|| {
//  let mut m = HashMap::new();
//  m.insert(13, "Spica".to_string());
//  m.insert(74, "Hoyten".to_string());
//  RwLock::new(m)
//});

#[derive(Debug, Clone)]
enum LanguageClientError {
  LanguageAlreadyDefined
}

#[derive(Debug, Clone)]
struct Languages {
  langs: HashMap<String, Value>
}

impl Languages {
  fn new() -> Self {
    Self { langs: HashMap::new() }
  }

  fn add_lang(&mut self, name: &String, content: &Value, force: bool) -> Result<(), LanguageClientError> {
    if let Some(lang) = self.langs.get_mut(&*name) {
      if force {
        *lang = content.clone();
        Ok(())
      } else {
        Err(LanguageClientError::LanguageAlreadyDefined)
      }
    } else {
      self.langs.insert(name.clone(), content.clone());
      Ok(())
    }
  }
}

static LANGUAGES: Lazy<RwLock<Languages>> = Lazy::new(|| {
  let mut lang = Languages::new();

  let fr_default_content = include_str!("../assets/languages/fr.json");
  let en_default_content = include_str!("../assets/languages/en.json");

  let fr_default = &serde_json::from_str(fr_default_content).unwrap();
  let en_default = &serde_json::from_str(en_default_content).unwrap();

  let _ = lang.add_lang(&"fr_default".to_string(), fr_default, true);
  let _ = lang.add_lang(&"en_default".to_string(), en_default, true);

  RwLock::new(lang)
});

//pub async fn load(locales_dir: &String){
//  
//  let mut lang_manager = LANGUAGES.write().await;
//
//  let locales_path = Path::new(locales_dir);
//  if !locales_path.is_dir() || !locales_path.exists() {
//    utils::error("LanguageHandler", "Cannot get directory to locales files", "Path doesn't exist or is not a valid directory");
//    exit(5);
//  };
//
//  for dir in fs::read_dir(locales_path).unwrap() {
//    let path = dir.unwrap().path();
//    let _ = fs::read_to_string(&path)
//      .map_err(|err| {
//        utils::error("LanguageHandler", "Fail to load language file", err.to_string().as_str());
//      })
//      .and_then(|cnt| Ok({
//
//        match path.file_stem().unwrap().to_str() {
//          Some(lang_name) => {
//            let try_json: Result<Value, _> = serde_json::from_str(cnt.as_str());
//            match try_json {
//              Ok(json) => {
//                let _ = lang_manager.add_lang(&lang_name.to_string(), &json, false);
//                utils::info("LanguageHandler", format!("Language file {} loaded successfully", lang_name).as_str());
//              },
//              Err(err) => {
//                utils::error("LanguageHandler", format!("Fail to parse JSON in {:?}", path.to_str().unwrap_or("UNKNOWN")).as_str(), err.to_string().as_str());
//              }
//            }
//          },
//          _ => {
//            utils::error_without_cause("LanguageHandler", "Fail to get language name");
//          }
//        }
//        
//      }));
//  };
//}

//macro_rules! format_obj {
//  ($($path: expr, $args:tt),*) => {{
//      format!($($args),*)
//  }};
//}
//
//fn translate(path: &str, args: HashMap<&str, &str>){
//
//}

pub async fn load(_: &String){}

pub async fn test(){
  let langs = LANGUAGES.read().await;
  dbg!(&langs.langs);
}