use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;
use std::time::{Duration, Instant};

use async_std::sync::Arc;
use async_std::task::{block_on, spawn_blocking};
use sysfs_class::{HwMon, SysClass};

use crate::adc::Measurement;
use crate::broker::{BrokerBuilder, Topic};

const UPDATE_INTERVAL: Duration = Duration::from_millis(500);

pub struct Temperatures {
    pub soc_temperature: Arc<Topic<Measurement>>,
    run: Option<Arc<AtomicBool>>,
}

impl Temperatures {
    pub fn new(bb: &mut BrokerBuilder) -> Self {
        let run = Arc::new(AtomicBool::new(true));
        let soc_temperature = bb.topic_ro("/v1/tac/temperatures/soc");

        let run_thread = run.clone();
        let soc_temperature_thread = soc_temperature.clone();
        spawn_blocking(move || {
            while run_thread.load(Ordering::Relaxed) {
                let val = HwMon::new(&"hwmon0")
                    .unwrap()
                    .temp(1)
                    .unwrap()
                    .input()
                    .unwrap();

                let meas = Measurement {
                    ts: Instant::now(),
                    value: val as f32 / 1000.0,
                };

                block_on(soc_temperature_thread.set(meas));

                sleep(UPDATE_INTERVAL);
            }
        });

        Self {
            soc_temperature,
            run: Some(run),
        }
    }
}

impl Drop for Temperatures {
    fn drop(&mut self) {
        self.run.take().unwrap().store(false, Ordering::Relaxed);
    }
}
