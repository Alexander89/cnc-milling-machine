// eslint-disable-next-line no-use-before-define
import React, { useState } from 'react'
import { obs, StatusMessage } from '../services'
import { Controller } from '../widget/Controller'
import { Mode } from '../widget/Mode'
import { Position } from '../widget/Position'
import { useViewStyle } from './style'

export const MainView = () => {
  const [status, setStatus] = useState<StatusMessage>()
  const { main, devModeBanner, header, cardBox } = useViewStyle()

  obs('status$', setStatus)

  return (
    <div className={main}>
      {status && status.devMode && (
        <div className={devModeBanner}>running in dev-mode</div>
      )}
      <div className={header}>System monitoring / control</div>
      <div className={cardBox}>
        <div>
          <Position />
          <Controller />
        </div>
        <div>
          <Mode />
        </div>
      </div>
    </div>
  )
}
