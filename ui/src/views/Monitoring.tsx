import React, { useEffect, useState } from 'react'
import { createUseStyles } from 'react-jss'
import { MonitoringService, Position } from '../services/monitoring'

export const Monitoring = () => {
  const [pos, setPos] = useState<Position>()
  const { main } = useStyle()
  useEffect(() => {
    MonitoringService.mock.position$.subscribe(setPos)
  }, [])
  return <div className={main}>
    <div>Monitoring</div>
    <div>
      <div>Position</div>
      {pos && (
        <div>
          <div>X: {pos.x}</div>
          <div>Y: {pos.y}</div>
          <div>Z: {pos.z}</div>
        </div>
      )}
    </div>
    <div>
      <div>Current Job</div>
    </div>
  </div>
}

const useStyle = createUseStyles({
  main: {
    width: 200
  }
})
