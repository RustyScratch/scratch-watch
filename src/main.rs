use sb_sbity::target::SpriteOrStage;
use serde::Deserialize;

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::{env::var, sync::mpsc::channel};

#[derive(Debug, Deserialize)]
struct Config {
    result_path: PathBuf,
    import_path: PathBuf,
    sprite_name: String,
}

fn main() {
    use notify::Watcher;
    use std::io::Write;

    fn get_blocks<'a>(
        project: &'a sb_sbity::project::Project,
        sprite_name: &str,
    ) -> &'a sb_sbity::string_hashmap::StringHashMap<sb_sbity::block::Block> {
        let sprite = project
            .targets
            .iter()
            .find_map(|target| {
                if let SpriteOrStage::Sprite(sprite) = target {
                    if sprite.target.name == sprite_name {
                        Some(sprite)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .expect("finding sprite");
        &sprite.target.blocks
    }

    let cfg_path = var("SB_STALKER_CFG").expect("SB_STALKER_CFG environment");
    let mut cfg_file = File::open(cfg_path).unwrap();
    let mut cfg_content = String::new();
    cfg_file.read_to_string(&mut cfg_content).unwrap();
    let cfg: Config = toml::from_str(&cfg_content).unwrap();
    let Config {
        result_path,
        import_path,
        sprite_name,
    } = cfg;

    let (tx, rx) = channel();

    let mut watcher = notify::RecommendedWatcher::new(tx, notify::Config::default()).unwrap();
    watcher
        .watch(Path::new(&import_path), notify::RecursiveMode::NonRecursive)
        .unwrap();

    println!("start watching");
    for res in rx {
        match res {
            Ok(event) => {
                println!("{event:?}");
                if let notify::EventKind::Create(_) = event.kind {
                    let file = File::options().read(true).open(&import_path).unwrap();
                    let mut zip_read = zip::read::ZipArchive::new(file).unwrap();
                    let mut json_zip = zip_read.by_name("project.json").unwrap();

                    let mut json = String::new();
                    json_zip.read_to_string(&mut json).unwrap();

                    let scratch_project: sb_sbity::project::Project =
                        serde_json::from_str(&json).unwrap();

                    let to_print = get_blocks(&scratch_project, &sprite_name);
                    let to_print = serde_json::to_string_pretty(&to_print).unwrap();

                    let mut file = std::fs::File::options()
                        .write(true)
                        .truncate(true)
                        .create(true)
                        .open(&result_path)
                        .unwrap();
                    file.write(to_print.as_bytes()).unwrap();
                }
            }
            Err(e) => eprintln!("{e}"),
        }
    }
}
