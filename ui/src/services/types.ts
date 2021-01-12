import { ControllerCommand } from './controller'
import { ProgramCommand } from './program'
import { SettingsCommand } from './settings'

export type CncCommand = ControllerCommand | ProgramCommand | SettingsCommand
