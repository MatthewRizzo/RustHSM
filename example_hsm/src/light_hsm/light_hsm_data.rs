use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, RwLock},
};

pub type LightHsmDataRef = Arc<RwLock<LightHsmData>>;

pub enum LightAdjustment {
    Increase,
    Decrease,
}

/// Data shared between the controller and the states, but that should be
/// invisible to EVERYTHING outside of the HSM as a whole
pub struct LightHsmData {
    /// 0 = off, 100 = on
    /// Again...leaking this is a bad idea. It is only done here for testing/asserting
    pub(crate) light_percentage: u8,
    pub top_enter_called: u16,
    pub top_start_called: u16,
    pub top_exit_called: u16,
    pub on_enter_called: u16,
    pub on_start_called: u16,
    pub on_exit_called: u16,
    pub off_enter_called: u16,
    pub off_start_called: u16,
    pub off_exit_called: u16,
    pub dimmer_enter_called: u16,
    pub dimmer_start_called: u16,
    pub dimmer_exit_called: u16,
}

impl LightHsmData {
    /// 0 = off, 100 = on
    pub(crate) fn new(percentage: u8) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self {
            light_percentage: percentage,
            top_enter_called: Default::default(),
            top_start_called: Default::default(),
            top_exit_called: Default::default(),
            on_enter_called: Default::default(),
            on_start_called: Default::default(),
            on_exit_called: Default::default(),
            off_enter_called: Default::default(),
            off_start_called: Default::default(),
            off_exit_called: Default::default(),
            dimmer_enter_called: Default::default(),
            dimmer_start_called: Default::default(),
            dimmer_exit_called: Default::default(),
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

    pub(crate) fn clear_counts(&mut self) {
        self.top_enter_called = Default::default();
        self.top_start_called = Default::default();
        self.top_exit_called = Default::default();
        self.on_enter_called = Default::default();
        self.on_start_called = Default::default();
        self.on_exit_called = Default::default();
        self.off_enter_called = Default::default();
        self.off_start_called = Default::default();
        self.off_exit_called = Default::default();
        self.dimmer_enter_called = Default::default();
        self.dimmer_start_called = Default::default();
        self.dimmer_exit_called = Default::default();
    }

    pub(crate) fn none_called(&self) -> bool {
        let mut res = true;
        if self.top_enter_called > 0 {
            println!("top_enter_called {} times", self.top_enter_called);
            res = false;
        }
        if self.top_start_called > 0 {
            println!("top_start_called {} times", self.top_start_called);
            res = false;
        }
        if self.top_exit_called > 0 {
            println!("top_exit_called {} times", self.top_exit_called);
            res = false;
        }
        if self.on_enter_called > 0 {
            println!("on_enter_called {} times", self.on_enter_called);
            res = false;
        }
        if self.on_start_called > 0 {
            println!("on_start_called {} times", self.on_start_called);
            res = false;
        }
        if self.on_exit_called > 0 {
            println!("on_exit_called {} times", self.on_exit_called);
            res = false;
        }
        if self.off_enter_called > 0 {
            println!("off_enter_called {} times", self.off_enter_called);
            res = false;
        }
        if self.off_start_called > 0 {
            println!("off_start_called {} times", self.off_start_called);
            res = false;
        }
        if self.off_exit_called > 0 {
            println!("off_exit_called {} times", self.off_exit_called);
            res = false;
        }
        if self.dimmer_enter_called > 0 {
            println!("dimmer_enter_called {} times", self.dimmer_enter_called);
            res = false;
        }
        if self.dimmer_start_called > 0 {
            println!("dimmer_start_called {} times", self.dimmer_start_called);
            res = false;
        }
        if self.dimmer_exit_called > 0 {
            println!("dimmer_exit_called {} times", self.dimmer_exit_called);
            res = false;
        }
        res
    }
}
