// eslint-disable-next-line no-use-before-define
import React, { useState } from 'react'
import { createUseStyles } from 'react-jss'
import { AlertMsg, useAlert } from '../services'
import { Button } from './Button'

export const AlertBox = () => {
  const [msg, setMsg] = useState<AlertMsg | undefined>(undefined)
  useAlert(setMsg)

  const { alertBox, alertWrapper, header, message } = useStyle()

  if (msg === undefined) {
    return <></>
  }

  const ButtonArea = () => {
    if (msg.buttons && msg.buttons.length) {
      return (
        <div style={{ display: 'flex' }}>
          {msg.buttons.map((btn) => (
            <Button
              key={btn.text}
              onClick={() => {
                setMsg(undefined)
                btn.onClick()
              }}
            >
              {btn.text}
            </Button>
          ))}
        </div>
      )
    } else {
      return <Button onClick={() => setMsg(undefined)}>Ok</Button>
    }
  }

  return (
    <div className={alertWrapper}>
      <div className={alertBox}>
        <div className={header}>{msg.title || 'Alert'}</div>
        <div className={message}>{msg.message}</div>
        <ButtonArea />
      </div>
    </div>
  )
}

const useStyle = createUseStyles({
  alertWrapper: {
    zIndex: 1000,
    position: 'absolute',
    top: 0,
    bottom: 0,
    left: 0,
    right: 0,
    backgroundColor: '#00000040'
  },
  alertBox: {
    backgroundColor: 'white',
    border: 'solid 1px #b0b0b0',
    borderRadius: 10,
    padding: '25px 35px',
    margin: '25% auto',
    width: 350
  },
  header: {
    fontSize: '2em',
    fontWeight: '900',
    paddingBottom: 25,
    borderBottom: 'solid 1px #ccc'
  },
  message: {
    marginTop: 25,
    marginBottom: 25
  }
})
