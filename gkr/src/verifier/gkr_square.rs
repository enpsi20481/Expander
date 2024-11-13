use super::verify_sumcheck_step;
use arith::{Field, FieldSerde};
use ark_std::{end_timer, start_timer};
use circuit::{Circuit, CircuitLayer};
use config::{Config, GKRConfig};
use std::{io::Read, vec};
use sumcheck::{GKRVerifierHelper, VerifierScratchPad};
use transcript::Transcript;

#[allow(clippy::type_complexity)]
pub fn gkr_square_verify<C: GKRConfig, T: Transcript<C::ChallengeField>>(
    config: &Config<C>,
    circuit: &Circuit<C>,
    public_input: &[C::SimdCircuitField],
    claimed_v: &C::ChallengeField,
    transcript: &mut T,
    mut proof_reader: impl Read,
) -> (
    bool,
    Vec<C::ChallengeField>,
    Vec<C::ChallengeField>,
    Vec<C::ChallengeField>,
    C::ChallengeField,
)
where
    [(); C::DEGREE_PLUS_ONE]:,
{
    let timer = start_timer!(|| "gkr verify");
    let mut sp = VerifierScratchPad::<C>::new(config, circuit);

    let layer_num = circuit.layers.len();
    let mut rz = vec![];
    let mut r_simd = vec![];
    let mut r_mpi = vec![];

    for _ in 0..circuit.layers.last().unwrap().output_var_num {
        rz.push(transcript.generate_challenge_field_element());
    }
    log::trace!("rz {:?}", rz);

    for _ in 0..C::get_field_pack_size().trailing_zeros() {
        r_simd.push(transcript.generate_challenge_field_element());
    }
    log::trace!("r_simd {:?}", r_simd);

    // TODO: MPI support
    assert_eq!(
        config.mpi_config.world_size().trailing_zeros(),
        0,
        "MPI not supported yet"
    );
    for _ in 0..config.mpi_config.world_size().trailing_zeros() {
        r_mpi.push(transcript.generate_challenge_field_element());
    }

    let mut verified = true;
    let mut current_claim = *claimed_v;
    log::trace!("Starting claim: {:?}", current_claim);
    for i in (0..layer_num).rev() {
        let cur_verified;
        (cur_verified, rz, r_simd, r_mpi, current_claim) = sumcheck_verify_gkr_square_layer(
            config,
            &circuit.layers[i],
            public_input,
            &rz,
            &r_simd,
            &r_mpi,
            current_claim,
            &mut proof_reader,
            transcript,
            &mut sp,
            i == layer_num - 1,
        );
        verified &= cur_verified;
    }
    end_timer!(timer);
    (verified, rz, r_simd, r_mpi, current_claim)
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
#[allow(clippy::unnecessary_unwrap)]
fn sumcheck_verify_gkr_square_layer<C: GKRConfig, T: Transcript<C::ChallengeField>>(
    config: &Config<C>,
    layer: &CircuitLayer<C>,
    public_input: &[C::SimdCircuitField],
    rz: &[C::ChallengeField],
    r_simd: &Vec<C::ChallengeField>,
    r_mpi: &Vec<C::ChallengeField>,
    current_claim: C::ChallengeField,
    mut proof_reader: impl Read,
    transcript: &mut T,
    sp: &mut VerifierScratchPad<C>,
    is_output_layer: bool,
) -> (
    bool,
    Vec<C::ChallengeField>,
    Vec<C::ChallengeField>,
    Vec<C::ChallengeField>,
    C::ChallengeField,
)
where
    [(); C::DEGREE_PLUS_ONE]:,
{
    // GKR2 with Power5 gate has degree 6 polynomial
    let degree = 6;

    GKRVerifierHelper::prepare_layer(layer, &None, rz, &None, r_simd, r_mpi, sp, is_output_layer);

    let var_num = layer.input_var_num;
    let mut sum = current_claim;
    sum -= GKRVerifierHelper::eval_cst(&layer.const_, public_input, sp);

    let mut rx = vec![];
    let mut r_simd_var = vec![];
    let mut r_mpi_var = vec![];
    let mut verified = true;

    for i_var in 0..var_num {
        verified &= verify_sumcheck_step::<C, T>(
            &mut proof_reader,
            degree,
            transcript,
            &mut sum,
            &mut rx,
            sp,
        );
        log::trace!("x {} var, verified? {}", i_var, verified);
    }
    GKRVerifierHelper::set_rx(&rx, sp);

    for i_var in 0..C::get_field_pack_size().trailing_zeros() {
        verified &= verify_sumcheck_step::<C, T>(
            &mut proof_reader,
            degree,
            transcript,
            &mut sum,
            &mut r_simd_var,
            sp,
        );
        log::trace!("simd {} var, verified? {}", i_var, verified);
    }
    GKRVerifierHelper::set_r_simd_xy(&r_simd_var, sp);

    // TODO: nontrivial MPI support
    for _i_var in 0..config.mpi_config.world_size().trailing_zeros() {
        verified &= verify_sumcheck_step::<C, T>(
            &mut proof_reader,
            3,
            transcript,
            &mut sum,
            &mut r_mpi_var,
            sp,
        );
        // println!("{} mpi var, verified? {}", _i_var, verified);
    }
    GKRVerifierHelper::set_r_mpi_xy(&r_mpi_var, sp);

    let v_claim = C::ChallengeField::deserialize_from(&mut proof_reader).unwrap();

    sum -= v_claim * GKRVerifierHelper::eval_pow_1(&layer.uni, sp)
        + v_claim.exp(5) * GKRVerifierHelper::eval_pow_5(&layer.uni, sp);
    transcript.append_field_element(&v_claim);

    verified &= sum == C::ChallengeField::ZERO;

    (verified, rx, r_simd_var, r_mpi_var, v_claim)
}
