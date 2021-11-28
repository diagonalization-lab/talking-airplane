use clap::Clap;
use msfs::sim_connect::{
    data_definition, Period, SimConnect, SimConnectRecv, SIMCONNECT_OBJECT_ID_USER,
};
use std::cell::RefCell;
use std::rc::Rc;

mod moving_average;
mod voice_box;

use std::time::{Duration, Instant};
use voice_box::VoiceBox;

#[derive(Debug, Clap, Default)]
struct Opts {
    /// Warning when indicated speed drops below this value.
    #[clap(long)]
    minspeed_warning: Option<f64>,
    /// Warning when indicated speed exceeds this value.
    #[clap(long)]
    maxspeed_warning: Option<f64>,
    /// Warn when bank angle exceeds this value.
    #[clap(long)]
    maxbank_warning: Option<f64>,
    /// Warn when pitch angle exceeds this value.
    #[clap(long)]
    maxpitch_warning: Option<f64>,
    /// Notification after these many minutes elapse.
    #[clap(long)]
    minute_timer: Option<u64>,
    /// Discards calculated vertical speeds with magnitudes that exceed this value.
    #[clap(long, default_value = "1000.0")]
    vertical_speed_discard_threshold: f64,
    /// Average vertical speed lower than this value is considered a descent.
    #[clap(long, default_value = "-200.0")]
    descent_threshold: f64,
    /// Average vertical speed greater than this value is considered a climb.
    #[clap(long, default_value = "200.0")]
    climb_threshold: f64,
    /// Number of vertical speed measurements we consider to calculate average vertical speed.
    #[clap(long, default_value = "30")]
    vertical_speed_window_size: usize,
}

#[data_definition]
#[derive(Debug)]
struct Data {
    #[name = "INDICATED ALTITUDE"]
    #[epsilon = 0.01]
    altitude: f64,
    #[name = "AIRSPEED INDICATED"]
    #[epsilon = 0.01]
    airspeed_indicated: f64,
    #[name = "ATTITUDE INDICATOR BANK DEGREES"]
    #[epsilon = 0.01]
    bank_degrees: f64,
    #[name = "ATTITUDE INDICATOR PITCH DEGREES"]
    #[epsilon = 0.01]
    pitch_degrees: f64,
}

fn report_data(s: &mut AppState, data: &Data) {
    let now = Instant::now();
    let altitude_int = data.altitude.round() as i32;
    let last_altitude_thousands = (s.history.last_altitude as i32) / 1000;
    let altitude_thousands = altitude_int / 1000;
    let altitude_to_nearest_500 = altitude_int - (altitude_int % 500);

    // In feet per minute
    let instantaneous_vert_speed = (data.altitude - s.history.last_altitude)
        / now.duration_since(s.history.last_time).as_secs_f64()
        * 60.0;
    if instantaneous_vert_speed.abs() < s.opts.vertical_speed_discard_threshold {
        // Note that we do not update last time if we discard the V/S measurement.
        s.history.last_time = now;
        s.history.average_vert_speed.add(instantaneous_vert_speed);
    }
    let average_vert_speed = s.history.average_vert_speed.average;
    s.history.last_altitude = data.altitude;

    // After the calculations above, the continuous value measurements in s.history are current.
    // However, discrete states such as s.history.inferred_phase and minspeed_ever_exceeded have
    // not been updated.

    match s.history.timer {
        None => {}
        Some(t) => {
            if now >= t {
                s.history.say_or_suppress(&mut s.voice_box, "Timer elapsed");
                s.history.timer = None;
            }
        }
    }

    if let Some(maxspeed_warning) = s.opts.maxspeed_warning {
        if data.airspeed_indicated > maxspeed_warning {
            s.history.say_or_suppress(
                &mut s.voice_box,
                &format!(
                    "Airspeed too high: {} knots.",
                    data.airspeed_indicated as i32
                ),
            );
        }
    }

    if let Some(bank_warning) = s.opts.maxbank_warning {
        if data.bank_degrees.abs() > bank_warning {
            s.history.say_or_suppress(
                &mut s.voice_box,
                &format!("Bank angle too high: {} degrees.", data.bank_degrees as i32),
            );
        }
    }

    if let Some(pitch_warning) = s.opts.maxpitch_warning {
        if data.pitch_degrees.abs() > pitch_warning {
            s.history.say_or_suppress(
                &mut s.voice_box,
                &format!(
                    "Pitch angle too high: {} degrees.",
                    data.pitch_degrees as i32
                ),
            );
        }
    }

    // Minimum speed warning set
    if let Some(minspeed_warning) = s.opts.minspeed_warning {
        if s.history.minspeed_ever_exceeded == false {
            if data.airspeed_indicated > minspeed_warning {
                s.history.minspeed_ever_exceeded = true;
                println!("Minimum speed warning is now enabled.");
            }
        } else {
            if data.airspeed_indicated < minspeed_warning {
                s.history.say_or_suppress(
                    &mut s.voice_box,
                    &format!("Airspeed below {}", minspeed_warning),
                );
            }
        }
    }

    // 1,000 feet per second is either impossible, or just an initialization
    // error.
    match (
        &mut s.history.inferred_phase,
        average_vert_speed < s.opts.descent_threshold,
        average_vert_speed > s.opts.climb_threshold,
    ) {
        (FlightPhase::Cruise(_), true, _) => {
            s.history.inferred_phase = FlightPhase::Descent;
            s.voice_box.say("Started descent.");
        }
        (FlightPhase::Cruise(_), _, true) => {
            s.history.inferred_phase = FlightPhase::Climb;
            s.voice_box.say("Climbing.");
        }
        (FlightPhase::Cruise(cruise_phase), false, false) => {
            if (data.airspeed_indicated - cruise_phase.airspeed).abs() > 10.0
                || (data.altitude - cruise_phase.altitude).abs() > 500.0
            {
                cruise_phase.airspeed = data.airspeed_indicated;
                cruise_phase.altitude = data.altitude;
                s.voice_box.say(&format!(
                    "Now cruising at {} feet and {} knots.",
                    altitude_to_nearest_500,
                    data.airspeed_indicated.round() as i32
                ));
            } else {
                // Continuing cruise. Nothing to say.
            }
        }
        (_, false, false) => {
            s.history.inferred_phase = FlightPhase::Cruise(CruisePhase {
                airspeed: data.airspeed_indicated,
                altitude: data.altitude,
            });
            s.voice_box.say("Leveling off.");
        }
        _ => {
            // Strange readings:
        }
    }

    if last_altitude_thousands != altitude_thousands {
        match s.history.inferred_phase {
            FlightPhase::Cruise(_) => {}
            FlightPhase::Climb => {
                s.voice_box
                    .say(&format!("Passing {} thousand feet.", altitude_thousands));
            }
            FlightPhase::Descent => {
                s.voice_box.say(&format!(
                    "Passing {} thousand feet.",
                    last_altitude_thousands
                ));
            }
        }
    }
}

#[derive(Debug, PartialEq)]
enum FlightPhase {
    Cruise(CruisePhase),
    Climb,
    Descent,
}

#[derive(Debug, PartialEq)]
struct CruisePhase {
    airspeed: f64,
    altitude: f64,
}

struct History {
    last_altitude: f64,
    last_time: Instant,
    last_warning: Instant,
    average_vert_speed: moving_average::MovingAverage,
    minspeed_ever_exceeded: bool,
    inferred_phase: FlightPhase,
    timer: Option<Instant>,
}

impl History {
    fn say_or_suppress(&mut self, s: &mut VoiceBox, msg: &str) {
        let now = Instant::now();
        if now.duration_since(self.last_warning) > Duration::from_secs(5) {
            self.last_warning = now;
            s.say(msg);
        }
    }
}

struct AppState {
    history: History,
    opts: Opts,
    voice_box: VoiceBox,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts: Opts = Opts::parse();
    let state: Rc<RefCell<AppState>> = Rc::new(RefCell::new(AppState {
        history: History {
            timer: match opts.minute_timer {
                None => None,
                Some(mins) => Some(Instant::now() + Duration::from_secs(mins * 60)),
            },
            inferred_phase: FlightPhase::Cruise(CruisePhase {
                airspeed: 0.0,
                altitude: 0.0,
            }),
            last_altitude: 0.0,
            last_time: Instant::now(),
            last_warning: Instant::now(),
            minspeed_ever_exceeded: false,
            average_vert_speed: moving_average::MovingAverage::new(opts.vertical_speed_window_size),
        },
        voice_box: VoiceBox::default(),
        opts: opts,
    }));

    let mut sim = SimConnect::open("talking-airplane", move |sim, recv| match recv {
        SimConnectRecv::SimObjectData(event) => match event.dwRequestID {
            0 => {
                report_data(&mut state.borrow_mut(), event.into::<Data>(sim).unwrap());
            }
            _ => {}
        },
        _ => {}
    })?;

    sim.request_data_on_sim_object::<Data>(0, SIMCONNECT_OBJECT_ID_USER, Period::Second)?;

    loop {
        sim.call_dispatch()?;
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
