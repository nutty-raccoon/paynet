mod outputs;
pub use outputs::{
    Error as OutputsError, check_outputs_allow_multiple_units, check_outputs_allow_single_unit,
    process_outputs,
};
mod inputs;
pub use inputs::{Error as InputsError, process_melt_inputs, process_swap_inputs};
