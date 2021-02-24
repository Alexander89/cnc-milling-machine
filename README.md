# cnc-milling driver for GCode / MCode

This little driver runs on a RPi V 3B+ and is connected to a PCB board to control the steppers.

Until now, a gamepad is required to move the head around and select and start programs.

A Web UI is available on http://<pi>:1506

## RPi connector Board

Eagle drawings and cam files are available in the folder `PCB` (TODO). Reach me out, I still have some PCBs on stock.
(4x V1 bank, 1x V1 assembled and tested, 1x V2 blank)

### Rev1

- end switches inputs
- Dir + Pul outputs
- screw holes

you can use Z-Down end switch input to calibrate

### Rev2

- end switches inputs
- Dir + Pul outputs
- 2 pwm out (servo or GPIO out/in 3.3V!)
- external power input (jumper)
- i²C connector
- calibrate input
- more compact
- screw holes

## Software features

- Support for G0 G1 G2 G3 G21 G90 G91 M0 M1 M3 M4 M5
- Settings file
- Switch spindle on / off
- Motor ramp for speed up
- Web UI for remote control
- Manipulate GCode in UI
- Multi input directory live watcher for USB-Stick detection.
- Show progress in UI
- V1: Responsive web ui for mobile phones

### Bugfix

- **F** is now mm/min. (pref. mm/sec)

## Upcoming features

1. External input as Change Tool (setting external_input_enabled) ( M6 )
2. Move spindel in UI with virtual joystick
3. Better Responsive web ui for mobile phones

backlog

4. Emergency trigger (stop || stop and go up)
5. Drive to max dimensions for selected program
6. Show path in ui
7. Redesign UI and widget System

## supported gCode commands

| code | command                                              | example                                                 |
| ---- | ---------------------------------------------------- | ------------------------------------------------------- |
| G0   | rapid move to XYZ position                           | G0 X1.0 Y0.0 Z 2.0                                      |
| G1   | milling move to XYZ position with F speed            | G0 X1.0 Y1.0 Z -1.0 F1.2                                |
| G2   | CW circle around IJK til XYZ is reached with F speed | G2 I-11.302914 J-5.330242 K0.0 X1.177652 Y5.964921 Z0.0 |
| G3   | CCW G2                                               | G3 I-11.302914 J-5.330242 K0.0 X1.177652 Y5.964921 Z0.0 |
| G21  | unit is mm                                           | G21                                                     |
| G90  | X, Y, Z Absolute                                     | G90                                                     |
| G91  | X, Y, Z Relative                                     | G91                                                     |
| M0   | Switch Off                                           | M0                                                      |
| M1   | Switch Off                                           | M1                                                      |
| M3   | Switch On                                            | M3                                                      |
| M4   | Switch On                                            | M4                                                      |
| M5   | Switch Off                                           | M5                                                      |
| M6   | Change tool (WIP)                                    | M6 T02                                                  |
