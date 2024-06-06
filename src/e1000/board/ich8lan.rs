use super::Hw;
use crate::err::*;
use crate::KernelFunc;
use crate::MediaType;
use crate::E1000;
pub(crate) fn get_variants<K: KernelFunc>(base: &mut Hw<K>) -> Result {
    init_mac_params(base)?;

    Ok(())
}

fn init_mac_params<K: KernelFunc>(base: &mut Hw<K>) -> Result {
    base.phy.media_type = MediaType::media_type_copper;

    Ok(())
}
