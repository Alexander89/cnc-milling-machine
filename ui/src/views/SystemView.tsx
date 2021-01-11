// eslint-disable-next-line no-use-before-define
import React, { useState } from 'react'
import { obs, StatusMessage } from '../services'
import { System } from '../widget/System'
import { useViewStyle } from './style'

export const SystemView = () => {
  const [status, setStatus] = useState<StatusMessage>()
  const { main, devModeBanner, header, cardBox } = useViewStyle()

  obs('status$', setStatus)

  return <div className={main}>
    {status && status.devMode && <div className={devModeBanner}>running in dev-mode</div>}
    <div className={header} style={{ display: 'block' }}>System configuration <span style={{ color: 'red', fontSize: '20' }}>requires a system restart</span></div>
    <div className={cardBox}>
      <div>
        <System />
      </div>
    </div>
  </div>
}
