use super::{Derivation, DerivationHash, DisplayTable, File};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{ContentArrangement, Table};
use sha2::Digest;
use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::{fs, io};
use steel::SteelErr;
use steel::rvals::IntoSteelVal;
use steel::steel_vm::builtin::BuiltInModule;
use steel::steel_vm::register_fn::RegisterFn;
use steel_derive::Steel;

#[derive(Steel, Clone)]
pub enum HashMethod {
    Contents,
    Timestamp,
}

impl File {
    pub fn new(
        path: String,
        hash_method: HashMethod,
    ) -> Result<File, SteelErr> {
        let path = PathBuf::from(path);
        let hash = calculate_hash(&path, hash_method)?;
        Ok(File { hash, path })
    }
    pub fn as_derivation(&self) -> Derivation {
        Derivation::File(self.clone())
    }

    pub fn display(&self) -> DisplayTable {
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            //.set_width(40)
            .add_row(vec!["hash".to_string(), format!("{}", self.hash)])
            .add_row(vec!["path".to_string(), format!("{:?}", self.path)]);

        DisplayTable { table }
    }
}

fn calculate_hash(
    path: &Path,
    hash_method: HashMethod,
) -> std::io::Result<DerivationHash> {
    let mut hasher = sha2::Sha256::new();
    match hash_method {
        HashMethod::Contents => {
            // iterates through file, updating the hasher as it goes, allowing very
            // large files to be hashed if necessary, only the hash is stored in memory
            let file = fs::File::open(path)?;
            for l in io::BufReader::new(file).lines().map_while(Result::ok) {
                hasher.update(l)
            }
        }
        HashMethod::Timestamp => {
            hasher.update(format!("{:?}", std::fs::metadata(path)?.modified()?))
        }
    };

    let path_name: &str = match path.file_name() {
        Some(v) => v.to_str().unwrap_or("None"),
        None => "None",
    };

    hasher.update(format!("{:?}", path));

    let result = hasher.finalize();
    let hash = format!("{}-{:x}", path_name, result);
    Ok(DerivationHash(hash))
}

pub fn register_steel_functions(
    module: &mut BuiltInModule,
) -> Result<(), SteelErr> {
    module.register_type::<File>("File?");
    module.register_value(
        "File::HashContents",
        HashMethod::Contents {}.into_steelval()?,
    );
    module.register_value(
        "File::HashTimestamp",
        HashMethod::Timestamp {}.into_steelval()?,
    );
    module.register_fn("File::new", File::new);
    module.register_fn("File::as_derivation", File::as_derivation);
    Ok(())
}
