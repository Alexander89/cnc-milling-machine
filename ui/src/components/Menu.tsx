import React from 'react'
import { createUseStyles } from 'react-jss'

export const Menu = () => {
  const { main, header, bottoms } = useStyle()
  return <div className={main}>
    <div className={header}>Menu</div>
    <div className={bottoms}>
      <div>Monitoring</div>
      <div>Jobs</div>
      <div>Setup</div>
    </div>
  </div>
}

const useStyle = createUseStyles({
  main: {
    width: 200,
    backgroundColor: '#6d6db0',
    padding: '15px 0px 15px 15px',
    color: 'White'
  },
  header: {
    fontSize: '2em',
    marginBottom: 10,
  },
  bottoms: {
    fontSize: '1.2em',
    '& div': {
      padding: '15px 0px 15px 10px',
      backgroundColor: '#7777aa',
      marginBottom: 5,
      marginRight: 0,
    },
    '& div:hover': {
      backgroundColor: '#5e5eb0',
    }
  },
  bottomActive: {
    backgroundColor: '#3f3fa4',
  }
})
