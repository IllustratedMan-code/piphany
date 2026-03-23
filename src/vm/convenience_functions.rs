use steel::SteelVal;
use steel::steel_vm::register_fn::RegisterFn;
use steel::steel_vm::engine::Engine;

/// convert x gigabytes to y megabytes
fn gb(x: u64) -> u64{
    x * 1000
}

/// convert x hours to y minutes
fn hours(x: u64) -> u64{
    x * 64
}

/// helper function for registering steel functions
pub fn register_steel_functions(vm: &mut Engine){
    vm.register_fn("GB", gb);
    vm.register_fn("hours", hours);
}
