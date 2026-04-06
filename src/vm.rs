use super::derivation_graph::DerivationGraph;
use steel::{steel_vm::engine::Engine, SteelErr};
use super::config::Config;
mod tester_functions;
mod convenience_functions;

pub fn engine(config_path: Option<std::path::PathBuf>) -> Result<Engine,  SteelErr> {
    let mut vm = Engine::new();
    let  c = Config::new(config_path);
    c.register_params(&mut vm);
    DerivationGraph::init(&mut vm, c)?;
    convenience_functions::register_steel_functions(&mut vm);
    tester_functions::register_steel_functions(&mut vm);
    Ok(vm)
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::debug_utils::Runner;
    macro_rules! test_scm_file {
        ($file:expr) => {{
            let mut e = engine(None).unwrap();
            e.run_builtin_or_print_error(include_str!($file), $file).expect("Failed Test");
        }};
    }

    #[test]
    fn basic_interpolations() {
        test_scm_file!("steel-modules/tests/basic_interpolations.scm")
    }

    #[test]
    fn node_interpolation() {
        test_scm_file!("steel-modules/tests/node_interpolation.scm");

    }
    #[test]
    fn parameterized_derivation() {
        test_scm_file!("steel-modules/tests/parameterized_derivation.scm");

    }

    #[test]
    fn data_frame_with_column(){
        test_scm_file!("steel-modules/tests/data_frame_with_column.scm")
    }

    #[test]
    fn data_frame_subset(){
        test_scm_file!("steel-modules/tests/data_frame_subset.scm")
    }

}
