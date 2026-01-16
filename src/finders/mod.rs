// Tue Jan 13 2026 - Alex

pub mod lua_api;
pub mod roblox;
pub mod bytecode;
pub mod structures;
pub mod classes;
pub mod properties;
pub mod methods;
pub mod constants;
pub mod result;
pub mod fflags;

pub use result::{
    FinderResult, StructureOffsetResult, ClassResult,
    PropertyResult, MethodResult, ConstantResult,
    ConstantValue, CombinedResults
};
pub use roblox::RobloxFinders;

use crate::memory::{Address, MemoryReader};
use std::sync::Arc;

pub struct AllFinders {
    reader: Arc<dyn MemoryReader>,
    roblox_finders: RobloxFinders,
}

impl AllFinders {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        let roblox_finders = RobloxFinders::new(reader.clone());

        Self {
            reader,
            roblox_finders,
        }
    }

    pub fn find_all(&self, start: Address, end: Address) -> CombinedResults {
        let mut results = CombinedResults::new();

        for result in self.roblox_finders.find_all(start, end) {
            results.add_function(result);
        }

        let structure_results = structures::find_all_structures(self.reader.clone(), start, end);
        for result in structure_results {
            results.add_structure_offset(result);
        }

        let class_results = classes::find_all_classes(self.reader.clone(), start, end);
        for result in class_results {
            results.add_class(result);
        }

        let property_results = properties::find_all_properties(self.reader.clone(), start, end);
        for result in property_results {
            results.add_property(result);
        }

        let method_results = methods::find_all_methods(self.reader.clone(), start, end);
        for result in method_results {
            results.add_method(result);
        }

        let constant_results = constants::find_all_constants(self.reader.clone(), start, end);
        for result in constant_results {
            results.add_constant(result);
        }

        results
    }
}
