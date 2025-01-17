// This file is part of tacd, the LXA TAC system daemon
// Copyright (C) 2022 Pengutronix e.K.
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; either version 2 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along
// with this program; if not, write to the Free Software Foundation, Inc.,
// 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.

use std::iter::Iterator;
use std::thread::sleep;
use std::time::Duration;

use anyhow::Result;
use async_std::task::block_on;

use crate::adc::IioThread;

pub struct LineHandle {
    name: String,
}

impl LineHandle {
    pub fn set_value(&self, val: u8) -> Result<(), ()> {
        // This does not actually set up any IIO things.
        // It is just a hack to let adc/iio/demo_mode.rs
        // communicate with this function so that toggling an output
        // has an effect on the measured values.
        let iio_thread = block_on(IioThread::new()).unwrap();

        match self.name.as_str() {
            "OUT_0" => iio_thread.get_channel("out0-volt").unwrap().set(val != 0),
            "OUT_1" => iio_thread.get_channel("out1-volt").unwrap().set(val != 0),
            "IOBUS_PWR_EN" => {
                iio_thread
                    .clone()
                    .get_channel("iobus-curr")
                    .unwrap()
                    .set(val != 0);
                iio_thread.get_channel("iobus-volt").unwrap().set(val != 0);
            }
            "DUT_PWR_EN" => {
                iio_thread
                    .clone()
                    .get_channel("pwr-curr")
                    .unwrap()
                    .set(val == 0);
                iio_thread.get_channel("pwr-volt").unwrap().set(val == 0);
            }
            _ => {}
        }

        Ok(())
    }
}

pub struct LineEvent(u8);

impl LineEvent {
    pub fn event_type(&self) -> EventType {
        match self.0 {
            0 => EventType::FallingEdge,
            _ => EventType::RisingEdge,
        }
    }
}

pub struct LineEventHandle {}

impl LineEventHandle {
    pub fn get_value(&self) -> Result<u8, ()> {
        Ok(0)
    }
}

impl Iterator for LineEventHandle {
    type Item = Result<LineEvent, ()>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            sleep(Duration::from_secs(1000));
        }
    }
}

pub enum EventType {
    RisingEdge,
    FallingEdge,
}

#[allow(non_camel_case_types)]
pub enum EventRequestFlags {
    BOTH_EDGES,
}

#[allow(clippy::upper_case_acronyms)]
pub enum LineRequestFlags {
    OUTPUT,
    INPUT,
}

pub struct FindDecoy {
    name: String,
}

impl FindDecoy {
    pub fn request(&self, _: LineRequestFlags, initial: u8, _: &str) -> Result<LineHandle> {
        let line_handle = LineHandle {
            name: self.name.clone(),
        };

        line_handle.set_value(initial).unwrap();

        Ok(line_handle)
    }

    pub fn events(
        &self,
        _: LineRequestFlags,
        _: EventRequestFlags,
        _: &str,
    ) -> Result<LineEventHandle, ()> {
        Ok(LineEventHandle {})
    }
}

pub fn find_line(name: &str) -> Result<FindDecoy> {
    Ok(FindDecoy {
        name: name.to_string(),
    })
}
