import React from 'react'
import { render } from 'react-dom'
import { createUseStyles } from 'react-jss'
import { Menu } from './components/Menu'
import { Monitoring } from './views/Monitoring'

export const Main = () => {
  const { main } = useStyle()
  return <div className={main}>
    <Menu />
    <Monitoring />
  </div>
}

const useStyle = createUseStyles({
  main: {
    width: '100vw',
    height: '100vh',
    overflow: 'hidden',
    display: 'flex',
  }
})

render(<Main />, document.getElementById('root'))