use crate::{unpack_and_combine, ProverScratchPad};
use arith::{Field, SimdField};
use circuit::CircuitLayer;
use config::GKRConfig;
use polynomials::EqPolynomial;

use super::{power_gate::SumcheckPowerGateHelper, simd_gate::SumcheckSimdProdGateHelper};

pub(crate) struct SumcheckGkrSquareHelper<'a, C: GKRConfig>
where
    [(); C::DEGREE_PLUS_ONE]:,
{
    pub(crate) rx: Vec<C::ChallengeField>,
    pub(crate) r_simd_var: Vec<C::ChallengeField>,

    layer: &'a CircuitLayer<C>,
    sp: &'a mut ProverScratchPad<C>,
    rz0: &'a [C::ChallengeField],
    r_simd: &'a [C::ChallengeField],

    _input_var_num: usize,
    _output_var_num: usize,
    pub(crate) simd_var_num: usize,

    x_helper: SumcheckPowerGateHelper<{ C::DEGREE_PLUS_ONE }>,
    simd_helper: SumcheckSimdProdGateHelper,
}

impl<'a, C: GKRConfig> SumcheckGkrSquareHelper<'a, C>
where
    [(); C::DEGREE_PLUS_ONE]:,
{
    #[inline]
    pub(crate) fn new(
        layer: &'a CircuitLayer<C>,
        rz0: &'a [C::ChallengeField],
        r_simd: &'a [C::ChallengeField],
        sp: &'a mut ProverScratchPad<C>,
    ) -> Self {
        let simd_var_num = C::get_field_pack_size().trailing_zeros() as usize;

        SumcheckGkrSquareHelper {
            rx: vec![],
            r_simd_var: vec![],

            layer,
            sp,
            rz0,
            r_simd,

            _input_var_num: layer.input_var_num,
            _output_var_num: layer.output_var_num,
            simd_var_num,

            x_helper: SumcheckPowerGateHelper::new(layer.input_var_num),
            simd_helper: SumcheckSimdProdGateHelper::new(simd_var_num),
        }
    }

    #[inline]
    pub(crate) fn poly_evals_at_x(
        &self,
        var_idx: usize,
    ) -> [C::ChallengeField; C::DEGREE_PLUS_ONE] {
        let evals = self.x_helper.poly_eval_at::<C>(
            var_idx,
            &self.sp.v_evals,
            &self.sp.pow_poly_evals,
            &self.sp.add_poly_evals,
            &self.layer.input_vals,
            &self.sp.gate_exists_pow,
            &self.sp.gate_exists_add,
        );
        let mut simd_combined = [C::ChallengeField::zero(); C::DEGREE_PLUS_ONE];
        for (combined, simd_val) in simd_combined.iter_mut().zip(evals.iter()) {
            *combined = unpack_and_combine(simd_val, &self.sp.eq_evals_at_r_simd0);
        }
        simd_combined
    }

    #[inline]
    pub(crate) fn poly_evals_at_simd(
        &self,
        var_idx: usize,
    ) -> [C::ChallengeField; C::DEGREE_PLUS_ONE] {
        self.simd_helper.gkr2_poly_eval_at::<C>(
            var_idx,
            &self.sp.eq_evals_at_r_simd0,
            &self.sp.simd_var_v_evals,
            self.sp.add_poly_evals[0],
            self.sp.pow_poly_evals[0],
        )
    }

    #[inline]
    pub(crate) fn receive_x_challenge(&mut self, var_idx: usize, r: C::ChallengeField) {
        self.x_helper.receive_challenge::<C>(
            var_idx,
            r,
            &mut self.sp.v_evals,
            &mut self.sp.pow_poly_evals,
            &mut self.sp.add_poly_evals,
            &self.layer.input_vals,
            &mut self.sp.gate_exists_pow,
            &mut self.sp.gate_exists_add,
        );
        log::trace!("v_eval[0]:= {:?}", self.sp.v_evals[0]);
        self.rx.push(r);
    }

    #[inline]
    pub(crate) fn receive_simd_challenge(&mut self, var_idx: usize, r: C::ChallengeField) {
        self.simd_helper.receive_challenge::<C>(
            var_idx,
            r,
            &mut self.sp.eq_evals_at_r_simd0,
            &mut self.sp.simd_var_v_evals,
            &mut self.sp.simd_var_hg_evals,
        );
        self.r_simd_var.push(r);
    }

    #[inline(always)]
    pub(crate) fn vx_claim(&self) -> C::ChallengeField {
        self.sp.simd_var_v_evals[0]
    }

    #[inline]
    pub(crate) fn prepare_simd(&mut self) {
        EqPolynomial::<C::ChallengeField>::eq_eval_at(
            self.r_simd,
            &C::ChallengeField::one(),
            &mut self.sp.eq_evals_at_r_simd0,
            &mut self.sp.eq_evals_first_half,
            &mut self.sp.eq_evals_second_half,
        );
    }

    #[inline]
    pub(crate) fn prepare_simd_var_vals(&mut self) {
        self.sp.simd_var_v_evals = self.sp.v_evals[0].unpack();
    }

    #[inline]
    pub(crate) fn prepare_g_x_vals(&mut self) {
        let uni = &self.layer.uni; // univariate things like square, pow5, etc.
        let vals = &self.layer.input_vals;
        let eq_evals_at_rz0 = &mut self.sp.eq_evals_at_rz0;
        let gate_exists_pow = &mut self.sp.gate_exists_pow;
        let gate_exists_add = &mut self.sp.gate_exists_add;
        let pow_poly_evals = &mut self.sp.pow_poly_evals;
        let add_poly_evals = &mut self.sp.add_poly_evals;
        // hg_vals[0..vals.len()].fill(F::zero()); // FIXED: consider memset unsafe?
        unsafe {
            std::ptr::write_bytes(pow_poly_evals.as_mut_ptr(), 0, vals.len());
            std::ptr::write_bytes(add_poly_evals.as_mut_ptr(), 0, vals.len());
        }
        // gate_exists[0..vals.len()].fill(false); // FIXED: consider memset unsafe?
        unsafe {
            std::ptr::write_bytes(gate_exists_pow.as_mut_ptr(), 0, vals.len());
            std::ptr::write_bytes(gate_exists_add.as_mut_ptr(), 0, vals.len());
        }
        EqPolynomial::<C::ChallengeField>::eq_eval_at(
            self.rz0,
            &C::ChallengeField::one(),
            eq_evals_at_rz0,
            &mut self.sp.eq_evals_first_half,
            &mut self.sp.eq_evals_second_half,
        );

        for g in uni.iter() {
            match g.gate_type {
                12345 => {
                    pow_poly_evals[g.i_ids[0]] +=
                        C::challenge_mul_circuit_field(&eq_evals_at_rz0[g.o_id], &g.coef);
                    gate_exists_pow[g.i_ids[0]] = true;
                }
                12346 => {
                    add_poly_evals[g.i_ids[0]] +=
                        C::challenge_mul_circuit_field(&eq_evals_at_rz0[g.o_id], &g.coef);
                    gate_exists_add[g.i_ids[0]] = true;
                }
                _ => panic!("Unsupported gate type"),
            }
        }
    }
}
