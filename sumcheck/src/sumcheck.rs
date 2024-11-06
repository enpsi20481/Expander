use circuit::CircuitLayer;
use config::{GKRConfig, MPIConfig};
use transcript::Transcript;

use crate::{
    prover_helper::{SumcheckGkrSquareHelper, SumcheckGkrVanillaHelper},
    ProverScratchPad,
};

// FIXME
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn sumcheck_prove_gkr_layer<C: GKRConfig, T: Transcript<C::ChallengeField>>(
    layer: &CircuitLayer<C>,
    rz0: &[C::ChallengeField],
    rz1: &Option<Vec<C::ChallengeField>>,
    r_simd: &[C::ChallengeField],
    r_mpi: &[C::ChallengeField],
    alpha: Option<C::ChallengeField>,
    transcript: &mut T,
    sp: &mut ProverScratchPad<C>,
    mpi_config: &MPIConfig,
    is_output_layer: bool,
) -> (
    Vec<C::ChallengeField>,
    Option<Vec<C::ChallengeField>>,
    Vec<C::ChallengeField>,
    Vec<C::ChallengeField>,
) {
    let mut helper = SumcheckGkrVanillaHelper::new(
        layer,
        rz0,
        rz1,
        r_simd,
        r_mpi,
        alpha,
        sp,
        mpi_config,
        is_output_layer,
    );

    helper.prepare_simd();
    helper.prepare_mpi();

    // gkr phase 1 over variable x
    helper.prepare_x_vals();
    for i_var in 0..helper.input_var_num {
        let evals = helper.poly_evals_at_rx(i_var, 2);
        let r = mpi_config.transcript_io::<C::ChallengeField, T>(&evals, transcript);
        helper.receive_rx(i_var, r);
    }

    helper.prepare_simd_var_vals();
    for i_var in 0..helper.simd_var_num {
        let evals = helper.poly_evals_at_r_simd_var(i_var, 3);
        let r = mpi_config.transcript_io::<C::ChallengeField, T>(&evals, transcript);
        helper.receive_r_simd_var(i_var, r);
    }

    helper.prepare_mpi_var_vals();
    for i_var in 0..mpi_config.world_size().trailing_zeros() as usize {
        let evals = helper.poly_evals_at_r_mpi_var(i_var, 3);
        let r = mpi_config.transcript_io::<C::ChallengeField, T>(&evals, transcript);
        helper.receive_r_mpi_var(i_var, r);
    }

    let vx_claim = helper.vx_claim();
    transcript.append_field_element(&vx_claim);

    // gkr phase 2 over variable y
    if !layer.structure_info.max_degree_one {
        helper.prepare_y_vals();
        for i_var in 0..helper.input_var_num {
            let evals = helper.poly_evals_at_ry(i_var, 2);
            let r = mpi_config.transcript_io::<C::ChallengeField, T>(&evals, transcript);
            helper.receive_ry(i_var, r);
        }
        let vy_claim = helper.vy_claim();
        transcript.append_field_element(&vy_claim);
    }

    let rx = helper.rx;
    let ry = if !layer.structure_info.max_degree_one {
        Some(helper.ry)
    } else {
        None
    };
    let r_simd = helper.r_simd_var;
    let r_mpi = helper.r_mpi_var;

    (rx, ry, r_simd, r_mpi)
}

// FIXME
#[allow(clippy::needless_range_loop)] // todo: remove
pub fn sumcheck_prove_gkr_square_layer<C: GKRConfig, T: Transcript<C::ChallengeField>>(
    layer: &CircuitLayer<C>,
    rz0: &[C::ChallengeField],
    r_simd: &[C::ChallengeField],
    transcript: &mut T,
    sp: &mut ProverScratchPad<C>,
) -> (Vec<C::ChallengeField>, Vec<C::ChallengeField>) {
    const D: usize = 7;
    let mut helper = SumcheckGkrSquareHelper::new(layer, rz0, r_simd, sp);

    helper.prepare_simd();
    helper.prepare_g_x_vals();

    // x-variable sumcheck rounds
    for i_var in 0..layer.input_var_num {
        let evals: [C::ChallengeField; D] = helper.poly_evals_at_x(i_var);

        for deg in 0..D {
            transcript.append_field_element(&evals[deg]);
        }
        let r = transcript.generate_challenge_field_element();

        log::trace!("x i_var={} evals: {:?} r: {:?}", i_var, evals, r);

        helper.receive_x_challenge(i_var, r);
    }

    // Unpack SIMD witness polynomial evaluations
    helper.prepare_simd_var_vals();

    // SIMD-variable sumcheck rounds
    for i_var in 0..helper.simd_var_num {
        let evals = helper.poly_evals_at_simd(i_var);

        for deg in 0..D {
            transcript.append_field_element(&evals[deg]);
        }
        let r = transcript.generate_challenge_field_element();

        log::trace!("SIMD i_var={} evals: {:?} r: {:?}", i_var, evals, r);

        helper.receive_simd_challenge(i_var, r);
    }

    log::trace!("vx claim: {:?}", helper.vx_claim());
    transcript.append_field_element(&helper.vx_claim());

    (helper.rx, helper.r_simd_var)
}
