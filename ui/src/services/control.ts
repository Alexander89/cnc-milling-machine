
// -------------- Messages

// -------------- Commands

export type OnOffCommand = {
  cmd: 'control'
  action: 'onOff'
  on: boolean
}
export type ControlCommand = OnOffCommand

// -------------- Services
