// eslint-disable-next-line no-use-before-define
import React, { useState } from 'react'
import { obs, StatusMessage } from '../services'
import { Mode } from '../widget/Mode'
import { ProgramEditor } from '../widget/ProgramEditor'
import { ProgramMetaData } from '../widget/ProgramMetaData'
import { ProgramSelect } from '../widget/ProgramSelect'
import { useViewStyle } from './style'

export const ProgramView = () => {
  const [status, setStatus] = useState<StatusMessage>()
  const { main, devModeBanner, header, cardBox } = useViewStyle()

  obs('status$', setStatus)

  return <div className={main}>
    {status && status.devMode && <div className={devModeBanner}>running in dev-mode</div>}
    <div className={header}>Program Management</div>
    <div className={cardBox} style={{ alignItems: 'stretch', height: 'calc(100vh - 300px)' }}>
      <ProgramSelect />
      <ProgramEditor />
      <div>
        <Mode />
        <ProgramMetaData />
      </div>
    </div>
  </div>
}
