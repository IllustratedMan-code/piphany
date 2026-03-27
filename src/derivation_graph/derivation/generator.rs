use super::{DisplayTable, Generator, Derivation, DerivationHash};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{ContentArrangement, Table};
use steel::steel_vm::builtin::BuiltInModule;
use steel::steel_vm::register_fn::RegisterFn;



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
}

pub fn register_steel_functions(module: &mut BuiltInModule) {
    module.register_fn("Generator::into_derivation", Generator::into_derivation);

}
