//! Module for creating the process graph.
//! Processes are "compiled" into derivations which
//! form the nodes of the ProcessGraph
use std::{collections::HashMap, hash::Hash};
use steel::rvals::Custom;
use steel::rvals::{FromSteelVal, IntoSteelVal};
use steel::steel_vm::builtin::BuiltInModule;
use steel::steel_vm::engine::Engine;
use steel::steel_vm::register_fn::RegisterFn;
use steel::SteelErr;
use steel_derive::Steel;
pub mod derivation;
pub mod derivation_runner;
use super::config::Config;
use derivation::Derivation;
use derivation::DerivationHash;

/// Directed Acyclic Graph containing the derivation nodes
#[derive(Clone, Steel)]
pub struct DerivationGraph {
    pub nodes: HashMap<derivation::DerivationHash, derivation::Derivation>,
    pub outputs: Option<derivation::Derivation>,
    pub config: Config,
}

static OUT_PLACEHOLDER: &str = "0000000000000000000-outdir";

/// extracts the ProcessGraph object from the scheme vm
pub fn extract_graph(vm: &mut Engine) -> Result<DerivationGraph, SteelErr> {
    let vm_dag = vm.extract_value("DG::graph")?;
    DerivationGraph::from_steelval(&vm_dag)
}

impl DerivationGraph {
    /// Inject the DerivationGraph object into the given scheme vm
    pub fn init(vm: &mut Engine, config: Config) -> Result<(), SteelErr>{
        let dag = DerivationGraph {
            nodes: HashMap::<DerivationHash, derivation::Derivation>::new(),
            config,
            outputs: None
        };
        let mut module = BuiltInModule::new("DerivationGraph");

        module.register_type::<DerivationGraph>("DerivationGraph?");
        module.register_value(
            "graph",
            dag.clone().into_steelval()?,
        );

        derivation::register_steel_functions(&mut module);
        derivation::process::scriptstring::register_steel_functions(&mut module);
        derivation::process::register_steel_functions(&mut module);
        derivation::file::register_steel_functions(&mut module)?;
        derivation::output::register_steel_functions(&mut module);
        derivation::dataframe::register_steel_functions(&mut module);
        derivation::generator::register_steel_functions(&mut module);
        module.register_fn("node_count", DerivationGraph::node_count);
        module.register_fn("display_nodes", DerivationGraph::display_nodes);
        module.register_fn("add_output", DerivationGraph::add_outputs);
        module.register_fn("add_derivation", DerivationGraph::add_derivation);
        module.register_type::<Derivation>("Derivation?");
        module.register_value(
            "config",
            dag.config
                .clone()
                .into_steelval()?
        );
        module.register_value(
            "out-hash-placeholder",
            OUT_PLACEHOLDER
                .into_steelval()?
        );
        vm.register_module(module);
        vm.register_steel_module(
            "derivation".to_string(),
            include_str!("steel-modules/derivation.scm").to_string(),
        );
        vm.run(r#"(require "derivation")"#)?;
        Ok(())
    }

    pub fn add_derivation(
        &mut self,
        derivation: Derivation,
    ) -> Result<Derivation, InsertError<DerivationHash>> {
        let hash = derivation.hash();
        self.nodes.safe_insert(hash, derivation.clone())
    }

    pub fn node_count(&self) {
        println!("{}", self.nodes.len())
    }

    pub fn display_nodes(&self) {
        println!("{:?}", self.nodes)
    }

    pub fn add_outputs(&mut self, derivation: derivation::Output){
        self.outputs = Some(Derivation::Output(derivation));
    }

}


pub struct InsertError<K> {
    node_id: K,
}

impl<K: std::fmt::Display + 'static> Custom for InsertError<K> {}

impl<K: std::fmt::Display> std::fmt::Display for InsertError<K> {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> Result<(), std::fmt::Error> {
        write!(f, "Node already in graph: {}", self.node_id)
    }
}

trait SafeInsert<K, V> {
    fn safe_insert(&mut self, key: K, value: V) -> Result<V, InsertError<K>>;
}

impl<K: Eq + Hash, V: Clone> SafeInsert<K, V> for HashMap<K, V> {
    fn safe_insert(&mut self, key: K, value: V) -> Result<V, InsertError<K>> {
        match self.get(&key) {
            Some(_) => Err(InsertError { node_id: key }),
            None => {
                self.insert(key, value.clone());
                Ok(value)
            }
        }
    }
}
