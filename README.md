# cnc-milling driver for gcode

This little driver runs on a RPi V4 and is connected to a PCB board to control the steppers.

Until now, a gamepad is required to move the head around and select and start programs

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

## Upcoming features

1. switch spindle on / off
2. motor ramp for speed up
3. create UI 
4. emergency trigger (stop || stop and go up)
5. drive to max dimensions for selected program
6. manipulate GCode in UI
7. show path in ui
8. more commands

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