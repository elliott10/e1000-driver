use super::defines::*;
use super::err::*;
use super::regs::*;
use crate::b82571::DevSpec82571;
use crate::hw::*;
use crate::KernelFunc;

pub(crate) struct HwSemaphore<K>
where
    K: KernelFunc,
{
    hw: HwRc<K>,
}

impl<K: KernelFunc> HwSemaphore<K> {
    pub(crate) fn new(hw: HwRc<K>, dev_spec: &mut DevSpec) -> Result<Self> {
        let mut s = Self { hw };

        match dev_spec {
            DevSpec::E82571(dev) => s.get_hw_semaphore_82571(dev)?,
            DevSpec::E80003es2lan => todo!(),
            DevSpec::Ich8lan(_) => todo!(),
        }

        Ok(s)
    }

    fn get_hw_semaphore_82571(&mut self, dev: &mut DevSpec82571) -> Result {
        let hw = self.hw.as_mut();
        let mut sw_timeout = hw.nvm.word_size + 1;
        let fw_timeout = hw.nvm.word_size + 1;
        let mut swsm = 0;

        /* If we have timedout 3 times on trying to acquire
         * the inter-port SMBI semaphore, there is old code
         * operating on the other port, and it is not
         * releasing SMBI. Modify the number of times that
         * we try for the semaphore to interwork with this
         * older code.
         */
        if dev.smb_counter > 2 {
            sw_timeout = 1;
        }

        /* Get the SW semaphore */
        let mut i = 0;

        while i < sw_timeout {
            swsm = hw.er32(SWSM);
            if (swsm & SWSM_SMBI) == 0 {
                break;
            }
            hw.kernel.delay(50_000);
            i += 1;
        }

        if (i == sw_timeout) {
            error!("Driver can't access device - SMBI bit is set.\n");
            dev.smb_counter += 1;
        }
        /* Get the FW semaphore. */
        i = 0;
        for _ in 0..fw_timeout {
            swsm = hw.er32(SWSM);
            hw.ew32(SWSM, swsm | SWSM_SWESMBI);

            /* Semaphore acquired if bit latched */
            if (hw.er32(SWSM) & SWSM_SWESMBI) > 0 {
                break;
            }

            hw.kernel.delay(50_000);

            i += 1;
        }

        if (i == fw_timeout) {
            /* Release semaphores */
            self.put_hw_semaphore_82571();
            error!("Driver can't access the NVM");
            return Err(Error::Nvm("Driver can't access the NVM"));
        }

        Ok(())
    }

    fn put_hw_semaphore_82571(&self) {
        let mut swsm = self.hw.as_ref().er32(SWSM);
        swsm &= !(SWSM_SMBI | SWSM_SWESMBI);
        self.hw.as_ref().ew32(SWSM, swsm);
    }
}

impl<K: KernelFunc> Drop for HwSemaphore<K> {
    fn drop(&mut self) {
        match self.hw.as_ref().variant {
            crate::BoardVariant::B82571 =>self.put_hw_semaphore_82571(),
            crate::BoardVariant::Ich8lan => todo!(),
        }
    }
}
