use super::{extension_field::random_extension_field_tests, field::random_field_tests};
use crate::{BabyBearExt3, BabyBearExt4};

#[test]
fn test_field() {
    // Deg 3
    random_field_tests::<BabyBearExt3>("Baby Bear Ext3".to_string());
    random_extension_field_tests::<BabyBearExt3>("Baby Bear Ext3".to_string());

    // Deg 4
    random_field_tests::<BabyBearExt4>("Baby Bear Ext4".to_string());
    random_extension_field_tests::<BabyBearExt4>("Baby Bear Ext4".to_string());
}
