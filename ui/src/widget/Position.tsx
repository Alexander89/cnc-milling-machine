// eslint-disable-next-line no-use-before-define
import * as React from 'react'
import { obs, PositionMessage } from '../services'

export const Position = () => {
  const [pos, setPos] = React.useState<PositionMessage>()

  obs('position$', setPos)

  return (
    <div className="card">
      <div className="header">Position</div>
      <div className="content">
        {pos && (
          <div className="row">
            <div className="posValue">
              X<div>{(pos.x / 10).toFixed(2)} cm</div>
            </div>
            <div className="posValue">
              Y<div>{(pos.y / 10).toFixed(2)} cm</div>
            </div>
            <div className="posValue">
              Z<div>{(pos.z / 10).toFixed(2)} cm</div>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}
