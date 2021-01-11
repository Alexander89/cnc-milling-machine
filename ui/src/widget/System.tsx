// eslint-disable-next-line no-use-before-define
import * as React from 'react'
import { useContext, useState } from 'react'
import { createUseStyles } from 'react-jss'
import { Button } from '../components/Button'
import { ToggleButton } from '../components/ToggleButton'
import { AlertCtx, obs, ServiceCtx } from '../services'
import { useWidgetStyle } from './style'

export const System = () => {
  const [programName, setProgramName] = useState<string>()
  const [program, setProgram] = useState<string>()
  const [invertZ, setInvertZ] = useState(false)
  const [scale, setScale] = useState(1)
  const { card, header } = useWidgetStyle()
  const { progEditor } = useStyle()
  const service = useContext(ServiceCtx)
  const { publish } = useContext(AlertCtx)

  obs('loadProg$', p => {
    setProgram(p?.program)
    setProgramName(p?.programName)
    setInvertZ(p?.invertZ || false)
    setScale(p?.scale || 1)
  })
  obs('startProg$', s => publish({ message: `Program ${s.programName} started` }))

  const start = () => programName && service?.sendCommand({ cmd: 'program', action: 'start', programName, invertZ, scale })
  const save = () => programName && program && service?.sendCommand({ cmd: 'program', action: 'save', programName, program })
  const deleteProg = () => programName && service?.sendCommand({ cmd: 'program', action: 'delete', programName })

  return (
    <div className={card} style={{ width: 550 }}>
      <div className={header} style={{ display: 'flex' }}>
        Program: {programName || ''}
      </div>
      <div>
      </div>
      <div className={progEditor}>
        {program ? <textarea style={{ width: '100%', height: '100%' }} value={program} onChange={e => e.target.value !== program} /> : 'select Program'}
      </div>
      {program && (<>
        <div style={{ display: 'flex', justifyContent: 'space-between' }}>
          <ToggleButton onClick={setInvertZ} value={invertZ}>Invert-Z</ToggleButton>
          <div></div>
          <div></div>
        </div>
        <div style={{ display: 'flex', justifyContent: 'space-between' }}>
            <Button onClick={start}>start</Button>
            <Button onClick={save}>save</Button>
            <Button onClick={deleteProg}>delete</Button>
        </div>
      </>)}
    </div>
  )
}

const useStyle = createUseStyles({
  progEditor: {
    fontSize: '1.4em',
    lineHeight: '1.8em',
    backgroundColor: 'white',
    margin: '10px 0px',
    height: '45vh',
    overflow: 'auto',
    '& > div': {
      padding: 10
    }
  }
})
