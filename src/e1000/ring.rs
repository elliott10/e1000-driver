use super::KernelFunc;

pub(crate) struct Ring<K>
where
    K: KernelFunc,
{
    kernel: K,
}

impl<K> Ring<K>
where
    K: KernelFunc,
{
    pub(crate) fn new(kernel: K) -> Self {
        Self { kernel }
    }
}

impl<K> Drop for Ring<K>
where
    K: KernelFunc,
{
    fn drop(&mut self) {}
}
