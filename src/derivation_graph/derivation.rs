use comfy_table::Table;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use process::scriptstring::ScriptString;
use std::path::PathBuf;
use std::{collections::HashMap, hash::Hash};
use steel::steel_vm::builtin::BuiltInModule;
use steel::steel_vm::register_fn::RegisterFn;
use steel::{SteelVal, rvals::Custom};

pub mod dataframe;
pub mod evaluator;
pub mod file;
pub mod generator;
pub mod output;
pub mod process;
pub mod test;
use steel_derive::Steel;

// Derivation needs to be an enum with possible derivation types
// File derivation, metadata derivation, output derivation

#[derive(Debug, Clone)]
pub enum Derivation {
    Empty,
    Process(Process),
    File(File),
    Output(Output),
    Generator(Generator),
    Test(Test),
    DataframeCsv(DataframeCsv),
    DataframeDB(DataframeDB),
}

impl Derivation {
    pub fn hash(&self) -> DerivationHash {
        match self {
            Derivation::Empty => DerivationHash("Empty".into()),
            Derivation::Process(v) => v.hash.clone(),
            Derivation::File(v) => v.hash.clone(),
            Derivation::Output(v) => v.hash.clone(),
            Derivation::Generator(v) => v.hash.clone(),
            Derivation::Test(v) => v.hash.clone(),
            Derivation::DataframeCsv(v) => v.hash.clone(),
            Derivation::DataframeDB(v) => v.hash.clone(),
        }
    }
    pub fn inputs(self) -> Option<Vec<DerivationHash>> {
        match self {
            Derivation::Empty => None,
            Derivation::Process(v) => Some(v.inward_edges),
            Derivation::File(_) => None,
            Derivation::Output(v) => Some(v.inward_edges),
            Derivation::DataframeCsv(v) => Some(v.inward_edges),
            Derivation::DataframeDB(v) => Some(v.inward_edges),
            Derivation::Generator(v) => Some(v.process.inward_edges.clone()),
            Derivation::Test(v) => Some(v.inward_edges.clone()),
        }
    }
    pub fn outputs(self) -> Vec<DerivationHash> {
        let hash = self.hash().clone();
        match self {
            Derivation::DataframeCsv(csv) => {
                let mut v = csv.inward_edges;
                v.push(hash);
                v
            }
            Derivation::DataframeDB(db) => {
                let mut v = db.inward_edges;
                v.push(hash);
                v
            }
            _ => vec![self.hash()],
        }
    }

    pub fn display(&self) -> Result<DisplayTable, String> {
        match self {
            Derivation::Empty => Ok(DisplayTable::empty()),
            Derivation::Process(v) => Ok(v.display()),
            Derivation::File(v) => Ok(v.display()),
            Derivation::Output(v) => Ok(v.display()),
            Derivation::DataframeCsv(v) => v.frame.display(),
            Derivation::DataframeDB(v) => Ok(v.display()),
            Derivation::Generator(v) => Ok(v.display()),
            Derivation::Test(v) => Ok(v.display()),
        }
    }
}

pub fn register_steel_functions(module: &mut BuiltInModule) {
    module.register_type::<Derivation>("Derivation?");
    module.register_fn("Derivation::hash", Derivation::hash);
    module.register_fn("Derivation::display", Derivation::display);
    module.register_fn("Derivation::inputs", Derivation::inputs);
}

impl steel::rvals::Custom for Derivation {
    fn fmt(&self) -> Option<std::result::Result<String, std::fmt::Error>> {
        <DerivationHash as Custom>::fmt(&self.hash())
    }
}

impl std::fmt::Display for Derivation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.hash())
    }
}

#[derive(Debug, Clone, Steel)]
pub struct File {
    pub path: PathBuf,
    pub hash: DerivationHash,
}

#[derive(Debug, Clone, Steel)]
pub struct Output {
    pub hash: DerivationHash,
    pub inward_edges: Vec<DerivationHash>,
}

#[derive(Debug, Clone)]
pub struct Dataframe {
    // we might not need this to be a derivationtype, just need to pass the derivations to the format
    pub frame: polars::prelude::DataFrame,
}

#[derive(Debug, Clone)]
pub struct DataframeCsv {
    pub hash: DerivationHash,
    pub frame: Dataframe,
    pub inward_edges: Vec<DerivationHash>,
    pub delimiter: String,
    pub ext: String,
}

#[derive(Debug, Clone)]
pub enum DataframeDBFormat {
    Excel,
    Sql,
}

#[derive(Debug, Clone)]
pub struct DataframeDB {
    pub hash: DerivationHash,
    pub frames: Vec<Dataframe>,
    pub format: DataframeDBFormat,
    pub inward_edges: Vec<DerivationHash>,
}

#[derive(Debug, Clone, Steel)]
pub struct Generator {
    pub association: Option<DerivationHash>, // Only one Generator association can exist in a process, two Nones count as different
    pub hash: DerivationHash,
    pub generator_kind: GeneratorKind, // glob or process
    pub process: Process // steal input edges from here
    // freeze generator interpolations in process, add method to resolve the script with
    // generator values (i.e. *.txt)
}

#[derive(Debug, Clone, Steel)]
pub enum GeneratorKind{

}

#[derive(Debug, Clone)]
pub enum GeneratorAssociationOperation {
    Cross,
    Zip,
}

#[derive(Debug, Clone, Steel)]
pub struct GeneratorAssociation {
    pub hash: DerivationHash,
    pub generators: Vec<DerivationHash>,
    pub operation: GeneratorAssociationOperation,
}

#[derive(Debug, Clone, Steel)]
pub struct Test {
    pub hash: DerivationHash,
    pub inward_edges: Vec<DerivationHash>,
}

/// Process Derivation
#[derive(Clone)] // Debug and Steel are custom implemented
pub struct Process {
    attributes: HashMap<String, SteelVal>,
    pub script: ScriptString,
    pub name: String,
    pub hash: DerivationHash,
    pub inward_edges: Vec<DerivationHash>,
    pub container: Option<String>,
    pub time: Option<usize>,
    pub memory: Option<usize>,
    pub shell: String,
    pub hpc_runtime: Option<String>,
    pub container_runtime: Option<String>,
    pub work_dir: String,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct DerivationHash(String);

impl std::fmt::Display for DerivationHash {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Custom for DerivationHash {
    fn fmt(&self) -> Option<std::result::Result<String, std::fmt::Error>> {
        Some(Ok(self.0.clone()))
    }
}

pub struct DisplayTable {
    table: Table,
}

impl DisplayTable {
    fn empty() -> Self {
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(comfy_table::ContentArrangement::Dynamic)
            .add_row(vec!["hash".to_string(), "Empty".into()]);
        Self { table }
    }
}

impl Custom for DisplayTable {
    fn fmt(&self) -> Option<std::result::Result<String, std::fmt::Error>> {
        Some(Ok(format!("\n{}", self.table)))
    }
}

impl std::fmt::Display for DisplayTable {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "\n{}", self.table)
    }
}

impl Default for Derivation {
    fn default() -> Self {
        Derivation::Empty
    }
}

impl polars_utils::total_ord::TotalHash for Derivation {
    fn tot_hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        state.write(self.hash().0.as_bytes())
    }
}
impl polars_utils::total_ord::TotalEq for Derivation {
    fn tot_eq(&self, other: &Self) -> bool {
        self.hash().0 == other.hash().0
    }
}
// required for PolarsObject
impl Hash for Derivation {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.hash().0.hash(state);
    }
}

impl PartialEq for Derivation {
    fn eq(&self, other: &Self) -> bool {
        self.hash() == other.hash()
    }
}

impl Eq for Derivation {}
