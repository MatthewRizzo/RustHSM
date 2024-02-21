use std::{cell::RefCell, rc::Rc};

pub type LightHsmDataRef = Rc<RefCell<LightHsmData>>;

pub enum LightAdjustment {
    Increase,
    Decrease,
}

/// Data shared between the controller and the states, but that should be
/// invisible to EVERYTHING outside of the HSM as a whole
pub struct LightHsmData {
    /// 0 = off, 100 = on
    /// Again...leaking this is a bad idea. It is only done here for testing/asserting
    pub (crate) light_percentage: u8,
}

impl LightHsmData {
    /// 0 = off, 100 = on
    pub(crate) fn new(percentage: u8) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            light_percentage: percentage,
        }))
    }

    pub(crate) fn turn_off(&mut self) {
        self.set_lighting(0);
    }

    pub(crate) fn turn_on(&mut self) {
        self.set_lighting(100);
    }

    /// 0 = off, 100 = on
    /// True on success
    pub(crate) fn set_lighting(&mut self, percentage: u8) -> bool {
        if percentage <= 100 {
            self.light_percentage = percentage;
            true
        } else {
            false
        }
    }

    /// Modifies the current lighting percentage modified by the provided %
    /// Bounded by [0-100]
    pub(crate) fn adjust_lighting_by_percentage(
        &mut self,
        percentage_modifier: u8,
        adjustment_type: LightAdjustment,
    ) {
        let adjustment_amount = self.light_percentage * percentage_modifier;
        let new_percentage = match adjustment_type {
            LightAdjustment::Decrease => self.light_percentage - adjustment_amount,
            LightAdjustment::Increase => self.light_percentage + adjustment_amount,
        };

        let bounded_adjustment = if new_percentage > 100 {
            100
        } else {
            new_percentage
        };
        self.set_lighting(bounded_adjustment);
    }
}
