use crate::time::{DateTime, Time};
use arduino_hal::port::mode::Output;
use arduino_hal::port::Pin;
use ufmt::derive::uDebug;
use ufmt::{uDebug, uWrite, uwrite};

pub struct Control<P1, P2, P3, P4> {
    pub start_time: DateTime,
    pub current_time: DateTime,
    pub ventil_gruppe: VentilGruppe<P1, P2, P3, P4>,
    pub control_mode: ControlMode,
    pub already_cleaned: bool,
    pub distance: Option<u16>,
    pub water_breach: Waterbreach,
}

impl<P1, P2, P3, P4> Control<P1, P2, P3, P4> {
    pub fn needs_cleaning(&self, time: Time) -> bool {
        if let ControlMode::Automatic(job, _) = &self.control_mode {
            if job == &Job::Idle
                && !self.already_cleaned
                && time.gt(&Time::from_hms(3, 0, 0))
                && time.le(&Time::from_hms(4, 0, 0))
            {
                return true;
            }
        }
        false
    }
}

impl<P1, P2, P3, P4> uDebug for Control<P1, P2, P3, P4>
    where
        P1: avr_hal_generic::port::PinOps,
        P2: avr_hal_generic::port::PinOps,
        P3: avr_hal_generic::port::PinOps,
        P4: avr_hal_generic::port::PinOps,
{
    fn fmt<W: ?Sized>(&self, f: &mut ufmt::Formatter<W>) -> Result<(), W::Error>
        where
            W: uWrite,
    {
        uwrite!(
            f,
            r#"{{
            "start_time": "{}",
            "current_time": "{}",
            "ventile": {:?},
            "mode": {:?},
            "distance": "{}",
            "water_breach": "{:?}"
        }}"#,
            self.start_time,
            self.current_time,
            self.ventil_gruppe,
            self.control_mode,
            self.distance.unwrap_or(0),
            self.water_breach
        )
    }
}

pub struct Waterbreach(pub Option<DateTime>);

impl uDebug for Waterbreach {
    fn fmt<W: ?Sized>(&self, f: &mut ufmt::Formatter<W>) -> Result<(), W::Error>
        where
            W: uWrite,
    {
        if self.0.is_some() {
            return uwrite!(f, "{}", self.0.unwrap());
        }
        uwrite!(f, "None")
    }
}

#[derive(PartialEq)]
pub enum ControlMode {
    Automatic(Job, Job),
    Manual(ManualControl),
    Breach,
    Off,
}

impl uDebug for ControlMode {
    fn fmt<W: ?Sized>(&self, f: &mut ufmt::Formatter<W>) -> Result<(), W::Error>
        where
            W: uWrite,
    {
        match self {
            ControlMode::Automatic(current_job, next_job) => {
                uwrite!(f,
            r#"{{
            "name": "{}",
            "jobs": [
                "{:?}",
                "{:?}"
            ]
            }}"#,
                "automatic", current_job, next_job)
            }
            ControlMode::Manual(manctrl) => {
                uwrite!(f,
            r#"{{
            "name": "{}",
            "jobs": [
                "{:?}"
            ]
            }}"#,
                "manual", manctrl)
            }
            ControlMode::Breach => {
                uwrite!(f,
            r#"{{
            "name": "{}",
            "jobs": []
            }}"#,
                "breach")
            }
            ControlMode::Off => {
                uwrite!(f,
            r#"{{
            "name": "{}",
            "jobs": []
            }}"#,
                "off")
            }
        }
    }
}

#[derive(uDebug, PartialEq)]
pub enum ManualControl {
    CurrentJob(Job),
    Bridged(bool, bool, bool, bool),
}

#[derive(PartialEq, uDebug)]
pub enum Job {
    Idle,
    Filter,
    Clean(DateTime),
}

pub struct VentilGruppe<P1, P2, P3, P4> {
    pub einlass: Ventil<P1>,
    pub abwasser: Ventil<P2>,
    pub filterwasser: Ventil<P3>,
    pub bridge: Ventil<P4>,
}

impl<P1, P2, P3, P4> VentilGruppe<P1, P2, P3, P4>
    where
        P1: avr_hal_generic::port::PinOps,
        P2: avr_hal_generic::port::PinOps,
        P3: avr_hal_generic::port::PinOps,
        P4: avr_hal_generic::port::PinOps,
{
    pub fn new(
        einlass: Pin<Output, P1>,
        abwasser: Pin<Output, P2>,
        filterwasser: Pin<Output, P3>,
        bridge: Pin<Output, P4>,
    ) -> Self {
        Self {
            einlass: Ventil::new(einlass),
            abwasser: Ventil::new(abwasser),
            filterwasser: Ventil::new(filterwasser),
            bridge: Ventil::new(bridge),
        }
    }

    pub fn set_clean(&mut self) {
        self.einlass.open();
        self.abwasser.open();
        self.filterwasser.close();
        self.bridge.open();
    }

    pub fn set_filter(&mut self) {
        self.einlass.open();
        self.abwasser.open();
        self.filterwasser.open();
        self.bridge.close();
    }

    pub fn set_idle(&mut self) {
        self.einlass.close();
        self.abwasser.close();
        self.filterwasser.close();
        self.bridge.close();
    }
}

impl<P1, P2, P3, P4> uDebug for VentilGruppe<P1, P2, P3, P4>
    where
        P1: avr_hal_generic::port::PinOps,
        P2: avr_hal_generic::port::PinOps,
        P3: avr_hal_generic::port::PinOps,
        P4: avr_hal_generic::port::PinOps,
{
    fn fmt<W: ?Sized>(&self, f: &mut ufmt::Formatter<W>) -> Result<(), W::Error>
        where
            W: uWrite,
    {
        uwrite!(
            f,
            r#"{{
            "einlass": {},
            "abwasser": {},
            "filterwasser": {},
            "bridge": {}
            }}"#,
            self.einlass.is_open(),
            self.abwasser.is_open(),
            self.filterwasser.is_open(),
            self.bridge.is_open()
        )
    }
}

pub struct Ventil<NR> {
    pin: Pin<Output, NR>,
    open: bool,
}

impl<NR> Ventil<NR>
    where
        NR: avr_hal_generic::port::PinOps,
{
    pub fn new(pin: Pin<Output, NR>) -> Self {
        Self { pin, open: false }
    }

    pub fn open(&mut self) {
        self.pin.set_high();
        self.open = true;
    }
    pub fn close(&mut self) {
        self.pin.set_low();
        self.open = false;
    }
    pub fn set(&mut self, b: bool) {
        if b {
            self.open();
        } else {
            self.close();
        }
    }
    pub fn is_open(&self) -> bool {
        self.open
    }
}
