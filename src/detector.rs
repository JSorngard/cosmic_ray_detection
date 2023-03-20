use std::cmp::Ordering;
use std::ptr::{read_volatile, write_volatile};

use rayon::prelude::*;

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

    /// Allocates more and more memory until is is not possible to do so anymore.
    /// Also fills that memory with the current value, or the default if there was no memory reserved.
    pub fn maximize_mass(&mut self) {
        const GIGA: usize = 1_000_000_000;

        if self.detector_mass.capacity() == 0 {
            // If you are using this program you probably have one kilobyte of memory.
            self.change_mass(1_000);
        }

        // Double capacity until it is no longer possible
        while self
            .detector_mass
            .try_reserve(self.detector_mass.capacity() * 2)
            .is_ok()
        {
            self.change_mass(self.detector_mass.capacity())
        }

        // Increment capacity by one gigabyte until it is no longer possible
        while self
            .detector_mass
            .try_reserve(self.detector_mass.capacity() + GIGA)
            .is_ok()
        {
            self.change_mass(self.detector_mass.capacity())
        }
    }

    /// Change the size of the detector.
    /// Also writes the current detector value to the new memory.
    pub fn change_mass(&mut self, new_capacity_bytes: usize) {
        match new_capacity_bytes.cmp(&self.detector_mass.capacity()) {
            Ordering::Greater => {
                let current_value = self.get(0).unwrap_or(self.default);

                // We do not know if this write is optimized away,
                // regardless of the value here.
                self.detector_mass.resize(new_capacity_bytes, 0);

                // So we guard against lazy memory paging here using the
                // volatile write functions.
                if current_value == 0 {
                    self.write(42);
                }
                self.write(current_value);
            }
            Ordering::Less => {
                self.detector_mass.truncate(new_capacity_bytes);
                self.detector_mass.shrink_to_fit();
            }
            Ordering::Equal => (),
        }
    }

    /// Checks if every element of the detector memory is equal to the default value.
    pub fn is_intact(&self) -> bool {
        self.find_index_of_changed_element().is_none()
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
    pub fn find_index_of_changed_element(&self) -> Option<usize> {
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
