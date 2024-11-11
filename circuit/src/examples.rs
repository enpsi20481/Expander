use crate::{Circuit, CircuitLayer, CoefType, GateAdd, GateConst, GateMul, GateUni};
use config::{GKRConfig, GKRScheme};

/// A circuit matching the example in the jupyter notebook
pub fn linear_gkr_test_circuit<C: GKRConfig>(_scheme: GKRScheme) -> Circuit<C> {
    let mut circuit = Circuit::default();

    // Layer 1
    let mut l1 = CircuitLayer {
        input_var_num: 2,
        output_var_num: 2,
        ..Default::default()
    };
    // node 0 * node 1
    l1.mul.push(GateMul {
        i_ids: [0, 1],
        o_id: 0,
        coef: C::CircuitField::from(1),
        coef_type: CoefType::Constant,
        gate_type: 1,
    });
    // + node 1
    l1.add.push(GateAdd {
        i_ids: [1],
        o_id: 1,
        coef: C::CircuitField::from(1),
        coef_type: CoefType::Constant,
        gate_type: 1,
    });
    // + node 1
    l1.add.push(GateAdd {
        i_ids: [1],
        o_id: 2,
        coef: C::CircuitField::from(1),
        coef_type: CoefType::Constant,
        gate_type: 1,
    });
    // + node 3
    l1.add.push(GateAdd {
        i_ids: [2],
        o_id: 2,
        coef: C::CircuitField::from(1),
        coef_type: CoefType::Constant,
        gate_type: 1,
    });
    // + node 4
    l1.add.push(GateAdd {
        i_ids: [3],
        o_id: 3,
        coef: C::CircuitField::from(1),
        coef_type: CoefType::Constant,
        gate_type: 1,
    });
    circuit.layers.push(l1);

    // Output layer
    let mut output_layer = CircuitLayer {
        input_var_num: 2,
        output_var_num: 1,
        ..Default::default()
    };
    // + 11 * node0
    output_layer.add.push(GateAdd {
        i_ids: [0],
        o_id: 0,
        coef: C::CircuitField::from(11),
        coef_type: CoefType::Constant,
        gate_type: 1,
    });
    // + node 1
    output_layer.add.push(GateAdd {
        i_ids: [1],
        o_id: 0,
        coef: C::CircuitField::from(1),
        coef_type: CoefType::Constant,
        gate_type: 1,
    });
    // + node 2
    output_layer.add.push(GateAdd {
        i_ids: [1],
        o_id: 1,
        coef: C::CircuitField::from(1),
        coef_type: CoefType::Constant,
        gate_type: 1,
    });
    // + node 3
    output_layer.add.push(GateAdd {
        i_ids: [2],
        o_id: 1,
        coef: C::CircuitField::from(1),
        coef_type: CoefType::Constant,
        gate_type: 1,
    });
    // + node 4
    output_layer.add.push(GateAdd {
        i_ids: [3],
        o_id: 1,
        coef: C::CircuitField::from(1),
        coef_type: CoefType::Constant,
        gate_type: 1,
    });
    circuit.layers.push(output_layer);

    circuit.identify_rnd_coefs();
    circuit
}

/// A simple GKR2 test circuit:
/// ```text
///         N_0_0     N_0_1             Layer 0 (Output)
///    x11 /   \    /    |  \
///  N_1_0     N_1_1  N_1_2  N_1_3      Layer 1
///     |       |    /  |      |   \
/// Pow5|       |  /    |      |    PI[0]
///  N_2_0     N_2_1   N_2_2  N_2_3     Layer 2 (Input)
/// ```
/// (Unmarked lines are `+` gates with coeff 1)
pub fn gkr_square_test_circuit<C: GKRConfig>() -> Circuit<C> {
    let mut circuit = Circuit::default();

    // Layer 1
    let mut l1 = CircuitLayer {
        input_var_num: 2,
        output_var_num: 2,
        ..Default::default()
    };
    // N_1_3 += PI[0] (public input)
    l1.const_.push(GateConst {
        i_ids: [],
        o_id: 3,
        coef: C::CircuitField::from(1),
        coef_type: CoefType::PublicInput(0),
        gate_type: 0,
    });
    // N_1_0 += (N_2_0)^5
    l1.uni.push(GateUni {
        i_ids: [0],
        o_id: 0,
        coef: C::CircuitField::from(1),
        coef_type: CoefType::Constant,
        gate_type: 12345,
    });

    // N_1_1 += N_2_1
    l1.uni.push(GateUni {
        i_ids: [1],
        o_id: 1,
        coef: C::CircuitField::from(1),
        coef_type: CoefType::Constant,
        gate_type: 12346,
    });
    // N_1_2 += N_2_1
    l1.uni.push(GateUni {
        i_ids: [1],
        o_id: 2,
        coef: C::CircuitField::from(1),
        coef_type: CoefType::Constant,
        gate_type: 12346,
    });
    // N_1_2 += N_2_2
    l1.uni.push(GateUni {
        i_ids: [2],
        o_id: 2,
        coef: C::CircuitField::from(1),
        coef_type: CoefType::Constant,
        gate_type: 12346,
    });
    // N_1_3 += N_2_3
    l1.uni.push(GateUni {
        i_ids: [3],
        o_id: 3,
        coef: C::CircuitField::from(1),
        coef_type: CoefType::Constant,
        gate_type: 12346,
    });
    circuit.layers.push(l1);

    // Output layer
    let mut output_layer = CircuitLayer {
        input_var_num: 2,
        output_var_num: 1,
        ..Default::default()
    };
    // N_0_0 += 11 * N_1_0
    output_layer.uni.push(GateUni {
        i_ids: [0],
        o_id: 0,
        coef: C::CircuitField::from(11),
        coef_type: CoefType::Constant,
        gate_type: 12346,
    });
    // N_0_0 += N_1_1
    output_layer.uni.push(GateUni {
        i_ids: [1],
        o_id: 0,
        coef: C::CircuitField::from(1),
        coef_type: CoefType::Constant,
        gate_type: 12346,
    });
    // N_0_1 += N_1_1
    output_layer.uni.push(GateUni {
        i_ids: [1],
        o_id: 1,
        coef: C::CircuitField::from(1),
        coef_type: CoefType::Constant,
        gate_type: 12346,
    });
    // N_0_1 += N_1_2
    output_layer.uni.push(GateUni {
        i_ids: [2],
        o_id: 1,
        coef: C::CircuitField::from(1),
        coef_type: CoefType::Constant,
        gate_type: 12346,
    });
    // N_0_1 += N_1_3
    output_layer.uni.push(GateUni {
        i_ids: [3],
        o_id: 1,
        coef: C::CircuitField::from(1),
        coef_type: CoefType::Constant,
        gate_type: 12346,
    });
    circuit.layers.push(output_layer);

    circuit.identify_rnd_coefs();
    circuit
}
