use crate::KernelFunc;
use crate::E1000;

impl<K> E1000<K>
where
    K: KernelFunc,
{
    pub(crate) fn get_variants_ich8lan(&mut self) {}
}