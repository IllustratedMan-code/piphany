use crate::derivation_graph::derivation::evaluator::HPCRuntimeFunctions;
use crate::derivation_graph::{
    DerivationGraph, derivation::Derivation, derivation::DerivationHash,
    derivation::Process,
};
use std::collections::VecDeque;

impl DerivationGraph {
    /// runs arbitrary derivation based on its hash
    pub fn run_derivation(
        &self,
        derivation_hash: DerivationHash,
    ) -> Result<(), String> {
        let mut run_order = VecDeque::<Vec<Derivation>>::new();
        let mut stop = false;
        let root = self
            .nodes
            .get(&derivation_hash)
            .ok_or("Derivation not in process graph".to_string());
        run_order.push_back(vec![root?.clone()]);
        while !stop {
            let mut iteration: Vec<Derivation> = Vec::new();
            let last_iter = run_order.back().expect(
                "no first element of run_order, this should never happen",
            );
            for i in last_iter {
                if let Some(edges) = i.clone().inputs() {
                    let mut derivations: Vec<Derivation> =
                        edges.iter().map(|edges| {
                            self.nodes.get(edges).expect("inward edges are not in process graph, this should never happen").clone()
                        }).collect();
                    iteration.append(&mut derivations);
                }
            }

            if iteration.is_empty() {
                stop = true
            } else {
                run_order.push_back(iteration);
            }
        }

        let mut i = 0;
        for iteration in run_order {
            let iteration: Vec<&Process> = iteration
                .iter()
                .filter_map(|x| match x {
                    Derivation::Process(v) => Some(v),
                    _ => None,
                })
                .collect();
            if iteration.is_empty(){
                continue;
            }
            i += 1;
            println!(
                "run iteration: {}, {:?}",
                i,
                iteration
                    .iter()
                    .map(|v| v.hash.clone())
                    .collect::<Vec<DerivationHash>>()
            );
            let handles: Vec<
                Option<
                    crate::derivation_graph::derivation::evaluator::HPCRuntime,
                >,
            > = iteration
                .iter()
                .map(|derivation| derivation.run())
                .collect();

            for mut handle in &mut handles.into_iter().flatten() {
                handle.wait();
            }
        }

        Ok(())
    }

    /// runs outputs derivation
    pub fn run(&self) -> Result<(), String> {
        self.run_derivation(
            self.outputs.clone().ok_or( "No outputs node!")?.hash(),
        )?;
        // need to replace with custom error type for derivations

        todo!()
    }
}
