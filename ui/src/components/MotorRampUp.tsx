// eslint-disable-next-line no-use-before-define
import * as React from 'react'

const drawGrid = (canvas: HTMLCanvasElement, ctx: CanvasRenderingContext2D, maxSpeed: number, time: number, leftSpacing: number, bottomSpacing: number) => {
  const stepsX = (canvas.width - leftSpacing) / 8
  ctx.lineWidth = 0.4
  ctx.beginPath()
  for (let x = 0, t = time / 8; x <= canvas.width - leftSpacing; x += stepsX, t += time / 8) {
    ctx.moveTo(x + leftSpacing, 0)
    ctx.lineTo(x + leftSpacing, canvas.height - bottomSpacing)
    ctx.fillText(t.toFixed(2) + ' s', x + leftSpacing + canvas.width / 12, canvas.height)
  }
  ctx.stroke()

  const stepsV = (canvas.height - bottomSpacing) / 5 // (maxSpeed / canvas.height) * maxSpeed / 10
  ctx.lineWidth = 0.4
  ctx.beginPath()
  for (let y = 24, v = maxSpeed / 10; y <= canvas.height; y += stepsV, v += maxSpeed / 10) {
    ctx.moveTo(leftSpacing, canvas.height - y)
    ctx.lineTo(canvas.width, canvas.height - y)
    ctx.fillText(v.toFixed(0), 0, canvas.height - y - 10)
  }
  ctx.stroke()

  ctx.lineWidth = 0.5

  ctx.beginPath()
  ctx.moveTo(0, 0)
  ctx.lineTo(time, 0)
  ctx.moveTo(0, 0)
  ctx.lineTo(0, 1)
  ctx.stroke()
}
const reset = (canvas: HTMLCanvasElement, ctx: CanvasRenderingContext2D) => {
  ctx.resetTransform()
  ctx.clearRect(0, 0, canvas.width, canvas.height)
  ctx.strokeStyle = '#000'
  ctx.scale(1, 1)
  ctx.font = '10px Arial'
}

type Props = {
  stepSize: number
  freeRunSpeed: number
  maxSpeed: number
  acceleration: number
  damping: number
  time: number
}

export const MotorRampUp = ({ stepSize, freeRunSpeed, maxSpeed, acceleration, damping, time } : Props) => {
  const canvasRef = React.useRef<HTMLCanvasElement>(null)

  const maxMotorSpeed = maxSpeed / stepSize / 60
  const startMotorSpeed = freeRunSpeed / stepSize / 60

  const leftSpacing = 40
  const bottomSpacing = 24

  React.useEffect(() => {
    const canvas = canvasRef.current
    if (canvas) {
      const ctx = canvas.getContext('2d')
      if (!ctx) {
        console.log('what old shitty browser are you using?')
        return
      }

      reset(canvas, ctx)
      drawGrid(canvas, ctx, maxMotorSpeed, time, leftSpacing, bottomSpacing)

      const f = (vLast: number) : number => {
        const v = Math.max(startMotorSpeed, vLast)
        return v + acceleration - v * damping
      }
      const dT = (v: number) : number => {
        if (v === 0) {
          return 0
        }

        // v == steps / sec
        // 1 / v == time one step take
        return 1 / v
      }

      const gridHight = canvas.height - bottomSpacing
      ctx.beginPath()
      ctx.moveTo(leftSpacing, gridHight)
      let last = 0
      let t = 0
      for (let x = 0; t < time; x++) {
        last = f(last)
        t += dT(last)
        ctx.lineTo((t / time) * canvas.width + leftSpacing, gridHight - (last / maxMotorSpeed) * gridHight)
      }
      ctx.stroke()
    }
  }, [canvasRef.current, acceleration, time, damping, startMotorSpeed, maxMotorSpeed])

  return (
    <div>
      <canvas ref={canvasRef} width="600" height="240" style={{ width: 600, height: 240, margin: '10px' }}/>
    </div>
  )
}
