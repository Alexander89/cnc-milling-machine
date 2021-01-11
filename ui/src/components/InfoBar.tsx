// eslint-disable-next-line no-use-before-define
import React, { useState } from 'react'
import { createUseStyles } from 'react-jss'
import { InfoMessage, obs } from '../services'
import { scan, filter } from 'rxjs/operators'

export const InfoBar = () => {
  const [messages, setMessage] = useState<Array<InfoMessage>>([])
  const { main, header, body } = useStyle()

  // @ts-ignore
  obs('info$', setMessage, filter(v => v !== undefined), scan<InfoMessage, Array<InfoMessage>>((acc, v) => [v, ...acc], []))

  return <div className={main}>
    <div className={header}>System info:</div>
    <div className={body}>
      {messages.map((msg, idx) => <div key={idx + msg.message}>{msg.lvl}: {msg.message}</div>)}
    </div>
  </div>
}

const useStyle = createUseStyles({
  main: {
    width: '100%',
    backgroundColor: '#6d6db0',
    padding: '10px 0px 15px 0px',
    color: 'white',
    height: 175
  },
  header: {
    fontSize: '1.5em',
    marginLeft: 15,
    marginBottom: 5
  },
  body: {
    height: 120,
    width: '100%',
    fontSize: '1em',
    backgroundColor: 'white',
    color: 'black',
    margin: 0,
    padding: 3,
    paddingLeft: 15,
    overflowX: 'hidden',
    overflowY: 'scroll',
    '& div': {
      lineHeight: '0.92em',
      marginRight: 0
    }
  }
})
