// eslint-disable-next-line no-use-before-define
import * as React from 'react'
import { createUseStyles } from 'react-jss'

type Props = {
  children: React.ReactNode
  onClick: () => void
}

export const Button = ({ children, onClick }: Props) => {
  const { buttonStyle } = useStyle()
  return (
    <div onClick={() => onClick()} className={buttonStyle}>
      <div>{children}</div>
    </div>
  )
}

const useStyle = createUseStyles({
  buttonStyle: {
    '& > div': {
      fontSize: '24px !important',
      backgroundColor: '#b0b0d7',
      border: 'solid 1px #d0d0d0',
      borderRadius: 10,
      boxShadow: '3px 3px 3px #555, inset 3px 3px 3px 0px #d6d6e6',
      adjustContent: 'center',
      textAlign: 'center',
      padding: '5px 10px',
      cursor: 'pointer'
    },
    '& > div:hover': {
      backgroundColor: '#a0a0d5',
      border: 'solid 1px #c0c0c0',
      boxShadow: '3px 3px 3px #555'
    }
  }
})
