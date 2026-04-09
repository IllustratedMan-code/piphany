use super::{DisplayTable, Generator, Derivation, DerivationHash, GeneratorKind};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{ContentArrangement, Table};
use steel::steel_vm::builtin::BuiltInModule;
use steel::steel_vm::register_fn::RegisterFn;
use sha2::Digest;


impl Generator {
    pub fn display(&self) -> DisplayTable{
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            //.set_width(40)
            .add_row(vec!["hash".to_string(), format!("{}", self.hash)]);

        DisplayTable{table}
    }

    pub fn into_derivation(self) -> Derivation{
        Derivation::Generator(self)
    }
    pub fn new_glob(process: super::Derivation, glob: String, regex: bool) -> Result<Self, String> {
        if let Derivation::Process(p) = process{
            let process = p;
        
            let mut hasher = sha2::Sha256::new();
            hasher.update(format!("{}{}{}", regex, glob, process.hash));
            let result = hasher.finalize();
            let hash = format!("generator-{}-{:x}", process.name, result);
            Ok(Self{
                association: None,
                hash: DerivationHash(hash),
                generator_kind: GeneratorKind::Glob{glob, regex},
                process
            })
        }else {
            Err("Input Derivation is of wrong type".into())
        }
    }
    pub fn new_process(process: super::Process) -> Self {
        let hash = DerivationHash(format!("generator-{}", process.hash.clone()));
        Self{
            association: None,
            hash,
            generator_kind: GeneratorKind::Process,
            process
        }
    }
}

pub fn register_steel_functions(module: &mut BuiltInModule) {
    module.register_fn("Generator::into_derivation", Generator::into_derivation);
    module.register_fn("Generator::new", Generator::new_glob);

}
