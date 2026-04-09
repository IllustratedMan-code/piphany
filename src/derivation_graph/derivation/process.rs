use super::DisplayTable;
use super::evaluator;
/// implementation for Process derivation
use super::{Derivation, DerivationHash, Generator, Process};
use crate::config::{Config, ParamValue};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{ContentArrangement, Table};
use scriptstring::ScriptString;
use sha2::Digest;
use std::collections::{HashMap, HashSet};
use steel::SteelErr;
use steel::{
    SteelVal,
    rvals::{Custom, FromSteelVal, IntoSteelVal},
};
pub mod scriptstring;
use steel::steel_vm::builtin::BuiltInModule;
use steel::steel_vm::register_fn::RegisterFn;
use steel_derive::Steel;

fn use_default_if_exists(
    default: HashMap<String, ParamValue>,
    values: HashMap<String, SteelVal>,
) -> HashMap<String, SteelVal> {
    let mut steel_defaults: HashMap<String, SteelVal> = default
        .iter()
        .map(|(k, v)| {
            (
                k.clone(),
                v.clone()
                    .into_steelval()
                    .expect("Couldn't convert param to steelval"),
            )
        })
        .collect();
    steel_defaults.extend(values);
    steel_defaults
}

fn calculate_hash(
    name: &String,
    script: &String,
    container: &Option<String>,
    shell: &String,
) -> DerivationHash {
    let container_string = container.clone().unwrap_or("".to_string());

    let mut hasher = sha2::Sha256::new();
    let combined = format!("{}{}{}{}", name, script, container_string, shell);
    hasher.update(combined);
    let result = hasher.finalize();
    let hash = format!("{:x}-{}", result, name); // {:x} works because of the LowerHex trait
    DerivationHash(hash)
}

macro_rules! extract_attribute {
    ($attributes:expr,$attr_name:literal, $target_type:ty) => {{
        let val = $attributes.get($attr_name);
        if let Some(v) = val {
            Some(<$target_type>::from_steelval(v)?)
        } else {
            None
        }
    }};
}

#[derive(Steel)]
enum AttributeError {
    Required(String),
}

impl std::fmt::Display for AttributeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttributeError::Required(v) => {
                write!(f, "Attribute {} is required in process definition", v)
            }
        }
    }
}

impl AttributeError {
    fn into_steel(self) -> SteelErr {
        SteelErr::new(
            steel::rerrs::ErrorKind::ContractViolation,
            format!("{}", self),
        )
    }
}

impl Process {
    pub fn new(
        attributes: HashMap<String, SteelVal>,
        config: Config,
    ) -> Result<Process, SteelErr> {
        // attributes unique to derivations

        let name = extract_attribute!(attributes, "name", String).ok_or_else(
            || AttributeError::Required("name".to_string()).into_steel(),
        )?;

        let script = extract_attribute!(attributes, "script", ScriptString);

        let script = extract_attribute!(attributes, "script", ScriptString)
            .ok_or_else(|| {
                AttributeError::Required("name".to_string()).into_steel()
            })?;

        // attributes from the config
        let merged_attributes =
            use_default_if_exists(config.config, attributes.clone());

        let time = extract_attribute!(merged_attributes, "time", usize);

        let memory = extract_attribute!(merged_attributes, "memory", usize);

        let shell = extract_attribute!(merged_attributes, "shell", String)
            .ok_or_else(|| {
                AttributeError::Required("shell".to_string()).into_steel()
            })?;

        let work_dir = extract_attribute!(merged_attributes, "workDir", String)
            .ok_or_else(|| {
                AttributeError::Required("workDir".to_string()).into_steel()
            })?;

        let container = None; // TODO need to add container handling

        let hash =
            calculate_hash(&name, &script.to_string(), &container, &shell);

        let d = Process {
            attributes: merged_attributes.clone(),
            hash,
            script: script.clone(),
            name,
            inward_edges: get_inward_edges(&script),
            container,
            time,
            memory,
            shell,
            hpc_runtime: None,
            container_runtime: None,
            work_dir,
            generators: get_generators(&script),
        };

        Ok(d)
    }
    pub fn as_derivation(self) -> Derivation {
        // if any generators exist in the interpolations, then
        // return a generator wrapping a process
        match self.generators {
            Some(_) => {
                return Generator::new_process(self).into_derivation();
            }
            None => return Derivation::Process(self),
        }
    }
    pub fn script(&self) -> String {
        self.script
            .to_string()
            .replace(super::super::OUT_PLACEHOLDER, &self.hash.to_string())
    }

    pub fn display(&self) -> DisplayTable {
        let mut table = Table::new();
        let hash = self.hash.clone();
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            //.set_width(40)
            .add_row(vec!["hash".to_string(), hash.0])
            .add_row(vec!["name".to_string(), self.name.clone()])
            .add_row(vec![
                "container".to_string(),
                self.container.clone().unwrap_or("None".to_string()),
            ])
            .add_row(vec!["shell".to_string(), self.shell.clone()])
            .add_row(vec!["script".to_string(), self.script()]);

        DisplayTable { table }
    }

    // TODO need to rewrite this to have its own method
    pub fn run(&self) -> Option<evaluator::HPCRuntime> {
        evaluator::run_derivation(self)
    }
}

pub fn register_steel_functions(module: &mut BuiltInModule) {
    module.register_type::<Process>("Process?");
    module.register_fn("Process::new", Process::new);
    module.register_fn("Process::as_derivation", Process::as_derivation);
}

// have to make custom type because can't implement external type for type from different crate
impl Custom for Process {
    fn fmt(&self) -> Option<std::result::Result<String, std::fmt::Error>> {
        Some(Ok(format!("{}", self.hash)))
    }
}

impl std::fmt::Display for Process {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.display())
    }
}

impl std::fmt::Debug for Process {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.attributes)
    }
}

fn extract_derivation_hashes(val: SteelVal) -> Vec<DerivationHash> {
    let mut vec = HashSet::<DerivationHash>::new();
    extract_derivation_hashes_recursive(val, &mut vec);
    vec.into_iter().collect()
}

fn extract_derivation_hashes_recursive(
    val: SteelVal,
    vec: &mut HashSet<DerivationHash>,
) {
    if let Ok(derivation) = Derivation::from_steelval(&val) {
        vec.extend(derivation.outputs());
        return;
    }

    if let Ok(vector) = Vec::<SteelVal>::from_steelval(&val) {
        for i in vector {
            extract_derivation_hashes_recursive(i, vec);
        }
        return;
    }

    if let Ok(hashmap) = HashMap::<SteelVal, SteelVal>::from_steelval(&val) {
        for (_, v) in hashmap {
            extract_derivation_hashes_recursive(v, vec)
        }
    }
}

fn get_inward_edges(script: &ScriptString) -> Vec<DerivationHash> {
    script
        .interpolations
        .iter()
        .flat_map(|i| extract_derivation_hashes(i.clone()))
        .collect::<HashSet<DerivationHash>>()
        .into_iter()
        .collect()
}

fn get_generators(script: &ScriptString) -> Option<Vec<Generator>> {
    let mut generators = vec![];
    for i in script.interpolations.iter() {
        if let Ok(Derivation::Generator(g)) = Derivation::from_steelval(i) {
            generators.push(g)
        }
    }

    if !generators.is_empty() {
        Some(generators)
    } else {
        None
    }
}
