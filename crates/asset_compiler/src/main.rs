#![feature(async_closure)]
#![feature(let_chains)]

use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::pin::Pin;
use std::process::Command;
use std::time::SystemTime;

use futures::Future;
use futures::executor::block_on;
use serde::Deserialize;
use serde::Serialize;

fn main() {
    block_on(async_main());
}

mod ktx {
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
    pub enum TargetType {
        #[default]
        RGBA,
        RGB,
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Timestamp {
    asset: SystemTime,
    meta:  SystemTime,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TextureMeta {
    encode:             String,
    #[serde(default)]
    is_normal:          bool,
    #[serde(default = "default_should_gen_mipmaps")]
    should_gen_mipmaps: bool,
    #[serde(default)]
    target_type:        ktx::TargetType,
}
fn default_should_gen_mipmaps() -> bool { true }

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct BuildCache(HashMap<String, Timestamp>);
impl BuildCache {
    pub const FILE: &str = "target/bifrons_asset_buildcache.ron";
}

fn visit_dirs(dir: &Path, buildcache: &mut BuildCache, futures: &mut Vec<Pin<Box<dyn Future<Output = ()>>>>) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, buildcache, futures)?;
            } else if let Some(filename) = path.file_name() {
                let name = filename.to_string_lossy().to_string();
                if name.ends_with(".ktx2.ron") {
                    let mut target_path = path.clone();
                    if let Some(filename) = name.strip_suffix(".ktx2.ron") {
                        target_path.set_file_name(filename)
                    } else { unreachable!() }
                    let target_path_str = target_path.to_string_lossy().to_string();

                    let mut outpath = target_path.clone();
                    outpath.set_extension("ktx2");

                    let meta_modified = {
                        let meta = std::fs::metadata(&path)?;
                        meta.modified()?
                    };
                    let target_modified = {
                        let meta = std::fs::metadata(&target_path)?;
                        meta.modified()?
                    };

                    if !outpath.is_file() || buildcache.0.get(&target_path_str).map(|m| meta_modified != m.meta || target_modified != m.asset).unwrap_or(true) {
                        println!("{} has changed; recompiling", target_path_str);
                        buildcache.0.insert(target_path_str, Timestamp {
                            asset: target_modified,
                            meta:  meta_modified,
                        });
                        futures.push(Box::pin((async move || {
                            let path      = path;
                            let mut f     = std::fs::File::open(&path).unwrap();
                            let mut bytes = Vec::new();
                            f.read_to_end(&mut bytes).unwrap();
                            let TextureMeta { encode, is_normal, should_gen_mipmaps, target_type } = ron_options().from_bytes(&bytes[..]).unwrap();
                            let target_path = target_path;
                            let outpath     = outpath;
                            let mut cmd = Command::new("toktx");
                            cmd.arg("--t2");
                            cmd.arg("--zcmp");
                            cmd.arg("--target_type");
                            cmd.arg(format!("{:?}", target_type));
                            if should_gen_mipmaps {
                                cmd.arg("--genmipmap");
                            }
                            if is_normal {
                                cmd.arg("--linear");
                                cmd.arg("--input_swizzle");
                                cmd.arg("rgb1");
                                cmd.arg("--normal_mode");
                            }
                            cmd.arg("--encode");
                            cmd.arg(encode);
                            cmd.arg(outpath.to_string_lossy().to_string());
                            cmd.arg(target_path.to_string_lossy().to_string());
                            
                            println!("Running {:?}", cmd);
                            let output = cmd.output().unwrap();
                            
                            io::stdout().write_all(&output.stdout).unwrap();
                            io::stderr().write_all(&output.stderr).unwrap();
                        })()));
                    }
                }
            }
        }
    }
    Ok(())
}

async fn async_main() {
    let cargo_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut bifrons_dir = PathBuf::from(&cargo_dir);
    bifrons_dir.pop();
    bifrons_dir.pop();
    let bc_path_str = format!("{}/{}", cargo_dir, BuildCache::FILE);
    let bc_path_buf  = PathBuf::from(&bc_path_str);

    let mut buildcache = {
        if bc_path_buf.is_file() {
            let bytes = match std::fs::read(&bc_path_buf) {
                Ok(file) => file,
                Err(e)   => panic!("Unable to open {}: {}", BuildCache::FILE, e),
            };
            match ron_options().from_bytes(&bytes) {
                Ok(file) => file,
                Err(e)   => panic!("Unable to deserialize {}: {}", BuildCache::FILE, e),
            }
        } else {
            BuildCache::default()
        }
    };

    let mut futures = Vec::new();
    visit_dirs(&Path::new(&bifrons_dir), &mut buildcache, &mut futures).unwrap();
    for f in futures {
        f.await
    }
    
    let cache_str = ron::ser::to_string_pretty(&buildcache, ron::ser::PrettyConfig::default()).unwrap();
    let mut file = match std::fs::File::create(bc_path_buf) {
        Ok(file) => file,
        Err(e)   => panic!("Unable to create {}: {}", BuildCache::FILE, e),
    };
    // ron bug? Its serialized output wraps in a (), which fails to deserialize.
    // So we have to strip the outer () in "({...})"
    if let Err(e) = file.write(cache_str[1..(cache_str.len() - 1)].as_bytes()) {
        panic!("Unable to write to {}: {}", BuildCache::FILE, e);
    }
}

pub fn ron_options() -> ron::Options {
    ron::Options::default().with_default_extension(ron::extensions::Extensions::all())
}