// eslint-disable-next-line no-use-before-define
import * as React from 'react'
import { Input } from '../Input'

type InputFieldProps = {
  title: string
} & (
  | {
      type: 'number'
      value: number | undefined
      defaultValue: number
      onChanged: (v: number) => void
    }
  | {
      type: 'string'
      value: string | undefined
      defaultValue: string
      onChanged: (v: string) => void
    }
)

export const InputField = (props: InputFieldProps) => {
  const onValueChanged = (v: string) => {
    props.type === 'number'
      ? props.onChanged(+v)
      : props.type === 'string'
        ? props.onChanged(v)
        // @ts-ignore
        : console.log('props.type not supported', props.type)
  }

  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'row',
        justifyContent: 'space-between'
      }}
    >
      {props.title}
      <div>
        <Input
          value={props.value || props.defaultValue}
          onChanged={onValueChanged}
        />
      </div>
    </div>
  )
}
