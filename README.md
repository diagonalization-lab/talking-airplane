Talking Airplane
================

Talking Airplane is a companion to Microsoft Flight Simulator that gives spoken
feedback when certain events occur. It is particularly handy when you are not
paying full attention, e.g., if you're running a long flight while doing other
things around the house. These are some of the events that it can report:

1. Reporting altitude during climb and descent.
2. Warnings when indicated airspeed exceeds or drops below a threshold.
3. Warnings when pitch or roll exceed a threshold.
4. Notification after a certain amount of time has elapsed.

**Note:** You must start Talking Airplane *before* you start Microsoft Flight
Simulator.

Talking Airplane is a command-line tool that takes several options:

```
USAGE:
    talking-airplane.exe [OPTIONS]

FLAGS:
    -h, --help       Print help information
    -V, --version    Print version information

OPTIONS:
        --climb-threshold <CLIMB_THRESHOLD>
            Average vertical speed greater than this value is considered a climb [default: 200.0]

        --descent-threshold <DESCENT_THRESHOLD>
            Average vertical speed lower than this value is considered a descent [default: -200.0]

        --maxbank-warning <MAXBANK_WARNING>
            Warn when bank angle exceeds this value

        --maxpitch-warning <MAXPITCH_WARNING>
            Warn when pitch angle exceeds this value

        --maxspeed-warning <MAXSPEED_WARNING>
            Warning when indicated speed exceeds this value

        --minspeed-warning <MINSPEED_WARNING>
            Warning when indicated speed drops below this value

        --minute-timer <MINUTE_TIMER>
            Notification after these many minutes elapse

        --vertical-speed-discard-threshold <VERTICAL_SPEED_DISCARD_THRESHOLD>
            Discards calculated vertical speeds with magnitudes that exceed this value [default:
            1000.0]

        --vertical-speed-window-size <VERTICAL_SPEED_WINDOW_SIZE>
            Number of vertical speed measurements we consider to calculate average vertical speed
            [default: 30]
```