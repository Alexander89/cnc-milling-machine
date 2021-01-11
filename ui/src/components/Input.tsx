// eslint-disable-next-line no-use-before-define
import * as React from 'react'
import { createUseStyles } from 'react-jss'

type Props = {
  width?: string | number
  value: string
  onChanged: (value: string) => void
  onBlur?: (value: string) => void
}

export const Input = ({ width, value, onChanged, onBlur }: Props) => {
  const { inputStyle } = useStyle()
  return <div className={inputStyle}>
    <input
      width={width}
      value={value}
      onChange={e => onChanged(e.target.value)}
      onBlur={e => onBlur && onBlur(e.target.value)} />
  </div>
}

const useStyle = createUseStyles({
  inputStyle: {
    '& > input': {
      fontSize: '24px !important',
      backgroundColor: '#f0f0f0',
      border: 'none',
      borderBottom: 'solid 2px #b0b0b8',
      borderBottomLeftRadius: 5,
      borderBottomRightRadius: 5,
      margin: '5px 10px',
      padding: '5px 10px',
      cursor: 'pointer'
    },
    '& > input:focus': {
      backgroundColor: '#ffffff',
      border: 'none',
      borderBottom: 'solid 3px #b0b0f8',
      outline: 'none'
    }
  }
})
