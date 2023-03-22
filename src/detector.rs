use std::ptr::{read_volatile, write_volatile};

#[cfg(not(windows))]
use crate::config::AllocationMode;

use rayon::prelude::{
    IndexedParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator,
};
use sysinfo::{RefreshKind, System, SystemExt};

/// In order to prevent the optimizer from removing the reads of the memory that make up the detector
/// this struct will only use volatile reads and writes to its memory.
pub struct Detector {
    parallel: bool,
    default: u8,
    detector_mass: Vec<u8>,
}

impl Detector {
    pub fn new(parallel: bool, default: u8, capacity_bytes: usize) -> Self {
        Detector {
            parallel,
            default,
            detector_mass: vec![default; capacity_bytes],
        }
    }

    #[cfg(windows)]
    /// Creates a new detector that fills up as much memory as possible.
    pub fn new_with_maximum_size(parallel: bool, default: u8) -> Self {
        // Know this is supported on windows.
        let s = System::new_with_specifics(RefreshKind::new().with_memory());
        let capacity_bytes = usize::try_from(s.available_memory())
            .expect("number of bytes of available memory fits in a usize");

        Detector {
            parallel,
            default,
            detector_mass: vec![default; capacity_bytes],
        }
    }

    #[cfg(not(windows))]
    /// Creates a new detector that fills up as much memory as possible in the specified way.
    /// # Panic
    /// Panics if this function is called on an operating system that is not supported by [sysinfo](https://crates.io/crates/sysinfo).
    pub fn new_with_maximum_size_in_mode(
        parallel: bool,
        default: u8,
        mode: AllocationMode,
    ) -> Self {
        if !<System as SystemExt>::IS_SUPPORTED {
            panic!("the current OS is not supported by the mechanism this program uses to determine available memory, please specify it manually");
        }

        let s = System::new_with_specifics(RefreshKind::new().with_memory());
        let capacity_bytes = usize::try_from(match mode {
            AllocationMode::Available => s.available_memory(),
            AllocationMode::Free => s.free_memory(),
        })
        .expect("number of bytes of available memory fits in a usize");

        Detector {
            parallel,
            default,
            detector_mass: vec![default; capacity_bytes],
        }
    }

    /// Returns the allocated memory size of the detector in bytes.
    pub fn capacity(&self) -> usize {
        self.detector_mass.capacity()
    }

    /// Checks if every element of the detector memory is equal to the default value.
    pub fn is_intact(&self) -> bool {
        self.position_of_changed_element().is_none()
    }

    /// Writes the given value to every element of the detector memory.
    pub fn write(&mut self, value: u8) {
        if self.parallel {
            self.detector_mass
                .par_iter_mut()
                .for_each(|n| unsafe { write_volatile(n, value) });
        } else {
            self.detector_mass
                .iter_mut()
                .for_each(|n| unsafe { write_volatile(n, value) });
        }
    }

    /// If an element in the detector does not match its default value, return it's index.
    pub fn position_of_changed_element(&self) -> Option<usize> {
        if self.parallel {
            self.detector_mass
                .par_iter()
                .position_any(|r| unsafe { read_volatile(r) != self.default })
        } else {
            self.detector_mass
                .iter()
                .position(|r| unsafe { read_volatile(r) != self.default })
        }
    }

    /// Resets the detector to its default value.
    pub fn reset(&mut self) {
        if self.default == 0 {
            // If some memory pages have been moved to swap due to inactivity
            // just writing zero to them might not prompt the OS to give them back.
            std::hint::black_box(self.write(42));
        }
        self.write(self.default);
    }

    /// Returns the value of the element at the given index, if it exists.
    pub fn get(&self, index: usize) -> Option<u8> {
        if index < self.detector_mass.len() {
            Some(unsafe { read_volatile(&self.detector_mass[index]) })
        } else {
            None
        }
    }
}
