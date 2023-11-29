/**
 * Generates a CSS class name based on the given string.
 *
 * @param {string} str - The input string for which the CSS class name is generated.
 * @param {boolean} [is32Bits=false] - If true, returns the last 4 characters of the generated hash.
 * @param {boolean} [is96Bits=false] - If true, returns the last 12 characters of the generated hash.
 * @returns {string} The generated CSS class name based on the input string, 
 * return the last 8 characters of the generated hash.
 */
module.exports = function generateCSSClassName(str) {
  var is32Bits = arguments.length > 1 && arguments[1] !== undefined ? arguments[1] : false;
  var is96Bits = arguments.length > 2 && arguments[2] !== undefined ? arguments[2] : false;
  /**
   * Map containing unique IDs for characters.
   * @type {Object.<string, string>}
   */
  var charsMap = {
    "A": "AbC123",
    "B": "DeF456",
    "C": "Ghi789",
    "D": "JkL012",
    "E": "MnO345",
    "F": "PqR678",
    "G": "StU901",
    "H": "VwX234",
    "I": "YzA567",
    "J": "BcD890",
    "K": "EfG123",
    "L": "HiJ456",
    "M": "KlM789",
    "N": "NoP012",
    "O": "PqR345",
    "P": "StU678",
    "Q": "VwX901",
    "R": "YzA234",
    "S": "BcD567",
    "T": "EfG890",
    "U": "HiJ123",
    "V": "KlM456",
    "W": "NoP789",
    "X": "PqR012",
    "Y": "StU345",
    "Z": "VwX678",
    "a": "ZyXWvU",
    "b": "TsRqPo",
    "c": "LkJiHg",
    "d": "DcB098",
    "e": "7h4P32",
    "f": "012zYx",
    "g": "UtSrQp",
    "h": "OnLkJi",
    "i": "HgFeDc",
    "j": "Ba0987",
    "k": "65w3X1",
    "l": "10gR76",
    "m": "2q45j7",
    "n": "89aI23",
    "o": "Zy27vU",
    "p": "Ts8mPo",
    "q": "L2J8H0",
    "r": "Dcj098",
    "s": "7A5s32",
    "t": "01K3p5",
    "u": "6789Ab",
    "v": "CdEfGh",
    "w": "IjKlMn",
    "x": "QrStUv",
    "y": "WxYz01",
    "z": "2Bh567",
    "0": "a2H129",
    "1": "D8x4P6",
    "2": "jIj0Y9",
    "3": "KFLw12",
    "4": "MnAi45",
    "5": "PqQp7b",
    "6": "StZi01",
    "7": "VCx0O4",
    "8": "8A95fS",
    "9": "BcDU8r",
    "!": "a1Rj23",
    "@": "fGw215",
    "#": "jLd87O",
    "$": "Pkl6ST",
    "%": "UrT57X",
    "^": "YQxg87",
    "&": "D8jloH",
    "*": "IkhT50",
    "(": "N0pYes",
    ")": "Tp5gbY",
    "-": "ZkT5sW",
    "_": "3zT5p8",
    "=": "Xhtx56",
    "+": "7pi6y0",
    "{": "a2NxEf",
    "}": "GhsP8L",
    "[": "M8bTkR",
    "]": "iTk678",
    ";": "VpTlYz",
    '"': "AwMAC4",
    "'": "5dOb7G",
    "<": "8o2DKl",
    ">": "M7RhPQ",
    ",": "R7umWn",
    ".": "Xp5V12",
    "?": "3xF6ab",
    "/": "c3RoEf",
    "|": "L89TnO",
    ":": "Q8YTuS"
  };
  /**
   * The based string to transform using unique character IDs.
   * @type {string}
   */
  var basedString = "";
  /**
   * The hash variable to store the generated hash.
   * @type {string}
   */
  var hash = "";
  try {
    // loops through each passed string characters
    // collects the unique id of each character
    // bind the unique id together to transform the based string
    for (var i = 0; i < str.length; i++) {
      basedString += charsMap[str[i]];
    }
    // loops through the based string with the unique ids
    // collects the char code from each character of it
    // gets the hex representation of each character
    // adds the hex representation to the hash variable
    for (var i = 0; i < basedString.length; i++) {
      var char = basedString.charCodeAt(i);
      hash += (char * 33).toString(16);
    }
    if (is32Bits) {
      // returns the last 4 characters
      return hash.slice(-4);
    } else if (is96Bits) {
      // returns the last 12 characters
      return hash.slice(-12);
    } else {
      // returns the last 8 characters
      return hash.slice(-8);
    }
  } catch (_unused) {}

  return "";
}