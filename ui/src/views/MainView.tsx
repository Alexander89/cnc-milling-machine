// eslint-disable-next-line no-use-before-define
import React, { useState } from 'react'
import { obs, StatusMessage } from '../services'
import { Controller } from '../widget/Controller'
import { Mode } from '../widget/Mode'
import { Position } from '../widget/Position'

export const MainView = () => {
  const [status, setStatus] = useState<StatusMessage>()
  obs('status$', setStatus)

  return (
    <div className="viewMain">
      {status && status.devMode && (
        <div className="viewDevModeBanner">running in dev-mode</div>
      )}
      <div className="viewHeader">System monitoring / control</div>
      <div className="viewCardBox">
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
