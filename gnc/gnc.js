define('vs/basic-languages/gnc/gnc', ["require", "exports"], function (require, exports) {
  "use strict"
  Object.defineProperty(exports, "__esModule", { value: true })
  exports.language = exports.conf = void 0
  exports.conf = {
    comments: {
      // symbol used for single line comment. Remove this entry if your language does not support line comments
      lineComment: ";",
      // symbols used for start and end a block comment. Remove this entry if your language does not support block comments
      blockComment: ["/*", "*/"]
    },
    // symbols used as brackets
    brackets: [
      ["{", "}"],
      ["[", "]"],
      ["(", ")"]
    ],
    // symbols that are auto closed when typing
    autoClosingPairs: [
      { open: '{', close: '}', notIn: ['string', 'comment'] },
      { open: '[', close: ']', notIn: ['string', 'comment'] },
      { open: '(', close: ')', notIn: ['string', 'comment'] },
      { open: '"', close: '"', notIn: ['string', 'comment'] },
      { open: "'", close: "'", notIn: ['string', 'comment'] }
    ],
    // symbols that that can be used to surround a selection
    surroundingPairs: [
      { open: '{', close: '}' },
      { open: '[', close: ']' },
      { open: '(', close: ')' },
      { open: '"', close: '"' },
      { open: "'", close: "'" }
    ]
  }
  exports.language = {
    defaultToken: '',
    tokenPostfix: '.gnc',
    // we include these common regular expressions
    escapes: /\\(?:[abfnrtv\\"']|x[0-9A-Fa-f]{1,4}|u[0-9A-Fa-f]{4}|U[0-9A-Fa-f]{8})/,
    tokenizer: {
      root: [
        { include: '@whitespace' },
        { include: '@comment' },
      ],
      keywords: [
        [/[nN][ \\t]*[0-9\\.]+/, 'constant.gcode'],
        [/[gG][ \\t]*[0-9][0-9\\.]*/, 'keyword.control.gcode.gcode'],
        [/(?i)[rRzZ][ \\t]*[\\-\\+]?[0-9\\.]+/, 'keyword.string.gcode'],
        [/(?i)[XAUI][ \\t]*[\\-\\+]?[0-9\\.]+/, 'string.xcode.gcode'],
        [/(?i)[YBVJ][ \\t]*[\\-\\+]?[0-9\\.]+/, 'comment.type.ycode.gcode'],
        [/[mM][ \\t]*[0-9][0-9\\.]*/, 'support.function.mcode.gcode'],
        [/[dDfFhHsStT][ \\t]*[0-9][0-9\\.]*/, 'support.type.dDfFhHsStTcode.gcode'],
        [/[pPqQeE][ \\t]*[0-9][0-9\\.]*/, 'variable.parameter.pqecode.gcode'],
        [/[cCwWkKlL][ \\t]*[\\-\\+]?[0-9\\.]+/, 'string.control.mcode.gcode'],
      ]
    }
  }
})
