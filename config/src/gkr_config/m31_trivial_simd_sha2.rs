//! A testing configuration where the SIMD field has packing size 1.
use super::{FiatShamirHashType, FieldType, GKRConfig};
use arith::ExtensionField;
use mersenne31::{M31Ext3, M31};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct M31TrivialSimdSha2;

impl GKRConfig for M31TrivialSimdSha2 {
    type CircuitField = M31;

    type ChallengeField = M31Ext3;

    type Field = M31Ext3; // SIMD is trivial

    type SimdCircuitField = M31; // SIMD is trivial

    const FIAT_SHAMIR_HASH: FiatShamirHashType = FiatShamirHashType::SHA256;

    const FIELD_TYPE: FieldType = FieldType::M31;

    fn challenge_mul_circuit_field(
        a: &Self::ChallengeField,
        b: &Self::CircuitField,
    ) -> Self::ChallengeField {
        a.mul_by_base_field(b)
    }

    fn field_mul_circuit_field(a: &Self::Field, b: &Self::CircuitField) -> Self::Field {
        a.mul_by_base_field(b)
    }

    fn field_add_circuit_field(a: &Self::Field, b: &Self::CircuitField) -> Self::Field {
        *a + *b
    }

    fn field_add_simd_circuit_field(a: &Self::Field, b: &Self::SimdCircuitField) -> Self::Field {
        a.add_by_base_field(b)
    }

    fn field_mul_simd_circuit_field(a: &Self::Field, b: &Self::SimdCircuitField) -> Self::Field {
        a.mul_by_base_field(b)
    }

    fn challenge_mul_field(a: &Self::ChallengeField, b: &Self::Field) -> Self::Field {
        let a_simd = Self::Field::from(*a);
        a_simd * b
    }

    fn circuit_field_into_field(a: &Self::SimdCircuitField) -> Self::Field {
        Self::Field::from(*a)
    }

    fn circuit_field_mul_simd_circuit_field(
        a: &Self::CircuitField,
        b: &Self::SimdCircuitField,
    ) -> Self::SimdCircuitField {
        Self::SimdCircuitField::from(*a) * b
    }

    fn circuit_field_to_simd_circuit_field(a: &Self::CircuitField) -> Self::SimdCircuitField {
        Self::SimdCircuitField::from(*a)
    }

    fn simd_circuit_field_into_field(a: &Self::SimdCircuitField) -> Self::Field {
        Self::Field::from(*a)
    }

    fn simd_circuit_field_mul_challenge_field(
        a: &Self::SimdCircuitField,
        b: &Self::ChallengeField,
    ) -> Self::Field {
        b.mul_by_base_field(&a).into()
    }
}
