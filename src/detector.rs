use std::ptr::{read_volatile, write_volatile};

#[cfg(all(not(target_os = "windows"), not(target_os = "freebsd")))]
use crate::config::AllocationMode;

#[cfg(feature = "rayon")]
use rayon::prelude::{
    IndexedParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator,
};

use sysinfo::{RefreshKind, System, SystemExt};

/// In order to prevent the optimizer from removing the reads of the memory that make up the detector
/// this struct will only use volatile reads and writes to its memory.
pub struct Detector {
    default: u8,
    detector_mass: Vec<u8>,
}

impl Detector {
    pub fn new(default: u8, capacity_bytes: usize) -> Self {
        Detector {
            default,
            detector_mass: vec![default; capacity_bytes],
        }
    }

    #[cfg(any(target_os = "windows", target_os = "freebsd"))]
    /// Creates a new detector that fills up as much memory as possible.
    pub fn new_with_maximum_size(default: u8) -> Self {
        // Know this is supported on windows.
        let s = System::new_with_specifics(RefreshKind::new().with_memory());
        let capacity_bytes = usize::try_from(s.available_memory())
            .expect("number of bytes of available memory fits in a usize");

        Detector {
            default,
            detector_mass: vec![default; capacity_bytes],
        }
    }

    #[cfg(all(not(target_os = "windows"), not(target_os = "freebsd")))]
    /// Creates a new detector that fills up as much memory as possible in the specified way.
    /// # Panic
    /// Panics if this function is called on an operating system that is not supported by [sysinfo](https://crates.io/crates/sysinfo).
    pub fn new_with_maximum_size_in_mode(default: u8, mode: AllocationMode) -> Self {
        if !<System as SystemExt>::IS_SUPPORTED {
            panic!("{} is not supported by the mechanism this program uses to determine available memory, please specify it manually", std::env::consts::OS);
        }

        let s = System::new_with_specifics(RefreshKind::new().with_memory());
        let capacity_bytes = usize::try_from(match mode {
            AllocationMode::Available => s.available_memory(),
            AllocationMode::Free => s.free_memory(),
        })
        .expect("number of bytes of available memory fits in a usize");

        Detector {
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
        #[cfg(feature = "rayon")]
        self.detector_mass
            .par_iter_mut()
            .for_each(|n| unsafe { write_volatile(n, value) });

        #[cfg(not(feature = "rayon"))]
        self.detector_mass
            .iter_mut()
            .for_each(|n| unsafe { write_volatile(n, value) });
    }

    /// If an element in the detector does not match its default value, return it's index.
    pub fn position_of_changed_element(&self) -> Option<usize> {
        #[cfg(feature = "rayon")]
        return self
            .detector_mass
            .par_iter()
            .position_any(|r| unsafe { read_volatile(r) != self.default });

        #[cfg(not(feature = "rayon"))]
        self.detector_mass
            .iter()
            .position(|r| unsafe { read_volatile(r) != self.default })
    }

    /// Resets the detector to its default value.
    pub fn reset(&mut self) {
        if self.default == 0 {
            // Just writing zero to memory pages might not prompt the OS to actually allocate them.
            // This is relevant the first time the detector is reset, and if the OS has moved
            // some pages to swap.
            self.write(42);
        }
        self.write(self.default);
    }

    /// Returns the value of the element at the given index, if it exists.
    pub fn get(&self, index: usize) -> Option<u8> {
        self.detector_mass
            .get(index)
            .map(|reference| unsafe { read_volatile(reference) })
    }
}
