// eslint-disable-next-line no-use-before-define
import * as React from 'react'
import { obs, PositionMessage } from '../services'
import { useWidgetStyle } from './style'

export const Position = () => {
  const [pos, setPos] = React.useState<PositionMessage>()

  const { card, header, row, posValue } = useWidgetStyle()
  obs('position$', setPos)

  return <div className={card}>
    <div className={header}>Position</div>
    {pos && (
      <div className={row}>
        <div className={posValue}>X<div>{(pos.x / 10).toFixed(2)} cm</div></div>
        <div className={posValue}>Y<div>{(pos.y / 10).toFixed(2)} cm</div></div>
        <div className={posValue}>Z<div>{(pos.z / 10).toFixed(2)} cm</div></div>
      </div>
    )}
  </div>
}
