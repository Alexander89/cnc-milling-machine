// eslint-disable-next-line no-use-before-define
import * as React from 'react'

export const MotorRampUp = () => {
  const canvasRef = React.useRef<HTMLCanvasElement>(null)

  React.useEffect(() => {
    if (canvasRef.current) {
      const ctx = canvasRef.current.getContext('2d')

      if (!ctx) {
        console.log('what old shitty browser are you using?')
        return
      }
      const maxSpeed = 200 // stepsPerSec
      const minDeltaT = 1 / maxSpeed
      const rampUpTime = 2 // sec
      ctx.lineWidth = 0.01

      ctx.translate(0, 100)
      ctx.scale(1, -100)

      ctx.beginPath()
      ctx.moveTo(0, 0)
      ctx.lineTo(1000, 0)
      ctx.moveTo(0, 0)
      ctx.lineTo(0, 1000)

      ctx.moveTo(0, 0)
      const _fNice = (x: number): number =>
        Math.min(Math.log(0.02 * Math.pow(x, 1.86) + 1.5) * 15, 100)

      const f = (x: number): number =>
        x / 1111.1111 + 0.1
      const f_inv = (y: number): number =>
        (y - 0.1) * 1111.1111
      for (let x = 0; x < 1000; x++) {
        ctx.lineTo(x, f(x))
      }
      ctx.stroke()

      ctx.beginPath()
      ctx.strokeStyle = '#0f0'
      ctx.moveTo(0, 0)
      for (let y = 0; y <= 1; y += 0.05) {
        ctx.lineTo(f_inv(y), y - 0.05)
        ctx.lineTo(f_inv(y), y)
      }
      ctx.stroke()

      ctx.beginPath()
      ctx.strokeStyle = '#f00'

      const deltaLastStep = 0.025 // sec
      const currentSpeed = 1 / deltaLastStep

      const minDelayNextStep = 0
      console.log(f(300))

      console.log(
        'maxSpeed', maxSpeed,
        'minDeltaT', minDeltaT,
        'rampUpTime', rampUpTime,
        'deltaLastStep', deltaLastStep,
        'currentSpeed', currentSpeed,
        'minDelayNextStep', minDelayNextStep)

      ctx.stroke()
    }
  }, [canvasRef.current])

  return (
    <div className="cardStretch RuntimeSettingsBox">
      <canvas ref={canvasRef} width="1000" height="100" style={{ width: 700, height: 200, margin: '10px' }}/>
    </div>
  )
}
