// eslint-disable-next-line no-use-before-define
import * as React from 'react'
import { useContext, useState } from 'react'
// import { createUseStyles } from 'react-jss'
import { Button } from '../components/Button'
import { Editor } from '../components/Editor'
import { Input } from '../components/Input'
import { ToggleButton } from '../components/ToggleButton'
import { AlertCtx, obs, ServiceCtx } from '../services'
import { useWidgetStyle } from './style'

export const ProgramEditor = () => {
  const [programName, setProgramName] = useState<string>()
  const [changedProg, setChangedProg] = useState<string | undefined>()
  const [newProg, setNewProg] = useState(false)
  const [program, setProgram] = useState<string>()
  const [invertZ, setInvertZ] = useState(false)
  const [scale, setScale] = useState(1)
  const { cardStretch, header, content } = useWidgetStyle()
  const service = useContext(ServiceCtx)
  const { publish } = useContext(AlertCtx)

  obs('loadProg$', (p) => {
    setNewProg(false)
    setProgram(p?.program)
    setProgramName(p?.programName)
    setInvertZ(p?.invertZ || false)
    setScale(p?.scale || 1)
  })
  obs('startProg$', (s) =>
    publish({ message: `Program ${s.programName} started` })
  )
  obs('saveProg$', (s) =>
    publish({ message: `Program ${s.programName} saved` })
  )
  obs('deleteProg$', (s) =>
    publish({ message: `Program ${s.programName} deleted` })
  )

  const create = () => {
    if (!programName || !service || !changedProg) {
      return
    }
    const name = programName.endsWith('.ngc')
      ? programName
      : `${programName}.ngc`
    service.sendCommand({
      cmd: 'program',
      action: 'save',
      programName: name,
      program: changedProg
    })
  }
  const start = () =>
    programName &&
    service?.sendCommand({
      cmd: 'program',
      action: 'start',
      programName,
      invertZ,
      scale
    })
  const save = () =>
    programName &&
    changedProg !== undefined &&
    service?.sendCommand({
      cmd: 'program',
      action: 'save',
      programName,
      program: changedProg
    })
  const deleteProg = () =>
    programName &&
    service?.sendCommand({ cmd: 'program', action: 'delete', programName })

  const addNew = () => {
    setNewProg(true)

    setProgram('')
    setProgramName(undefined)
    setInvertZ(false)
    setScale(1)
  }

  return (
    <div className={cardStretch} style={{ maxHeight: '100%', width: 560 }}>
      <div
        className={header}
        style={{ display: 'flex', justifyContent: 'space-between' }}
      >
        Program: {newProg ? '' : programName || ''}
        <div style={{ display: 'inline-block' }}>
          {newProg
            ? (
            <Input
              value={programName || ''}
              onChanged={setProgramName}
              width={20}
            />
              )
            : (
            <Button onClick={addNew}>New</Button>
              )}
        </div>
      </div>
      <div
        className={content}
        style={{ display: 'flex', flexDirection: 'column', height: '100%' }}
      >
        <div style={{ flex: '1', width: '100%' }}>
          <Editor
            style={{ flex: '1' }}
            file={
              program === undefined ? 'Select or create a program' : program
            }
            diff={undefined}
            onChanged={setChangedProg}
          />
        </div>
        <div style={{ flex: '0 0 100px' }}>
          {newProg && (
            <div style={{ display: 'flex', justifyContent: 'space-between' }}>
              <Button onClick={create}>Create</Button>
              <div></div>
            </div>
          )}
          {program && (
            <>
              <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                <ToggleButton onClick={setInvertZ} value={invertZ}>
                  Invert-Z
                </ToggleButton>
                <div></div>
                <div></div>
              </div>
              <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                <Button onClick={start}>Start</Button>
                <Button onClick={save}>Save</Button>
                <Button onClick={deleteProg}>Delete</Button>
              </div>
            </>
          )}
        </div>
      </div>
    </div>
  )
}

/*
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
}) */
