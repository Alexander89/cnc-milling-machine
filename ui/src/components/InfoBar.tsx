// eslint-disable-next-line no-use-before-define
import React, { useState } from 'react'
import { createUseStyles } from 'react-jss'
import { InfoMessage, obs } from '../services'
import { scan, filter } from 'rxjs/operators'
import { primary } from '../theme'

export const InfoBar = () => {
  const [messages, setMessage] = useState<Array<InfoMessage>>([])
  const { main, header, body } = useStyle()

  obs(
    'info$',
    // @ts-ignore
    setMessage,
    filter((v) => v !== undefined),
    scan<InfoMessage, Array<InfoMessage>>((acc, v) => [v, ...acc], [])
  )

  return (
    <div className={main}>
      <div className={header}>System info:</div>
      <div className={body}>
        {messages.map((msg, idx) => (
          <div key={idx + msg.message}>
            {msg.lvl}: {msg.message}
          </div>
        ))}
      </div>
    </div>
  )
}

const useStyle = createUseStyles({
  main: {
    width: '100%',
    backgroundColor: primary,
    padding: '12px 0px 12px 0px',
    height: 175
  },
  header: {
    fontWeight: 900,
    textTransform: 'uppercase',
    marginLeft: 12,
    marginBottom: 12
  },
  body: {
    height: 120,
    fontSize: '1em',
    backgroundColor: 'white',
    color: 'black',
    margin: 0,
    padding: 6,
    paddingLeft: 12,
    overflowX: 'hidden',
    overflowY: 'scroll',
    '& div': {
      lineHeight: '0.92em',
      marginRight: 0
    }
  }
})
