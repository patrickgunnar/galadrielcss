pub fn get_nenyr_syntax() -> String {
    r#"%YAML 1.2
---
name: Nenyr
scope: source.nenyr
file_extensions:
    - nyr
contexts:
    main:
        - include: keywords
        - include: scopes
        - include: declare
        - include: handlers
        - include: nested
        - include: properties
        - include: anime
        - include: boolean
        - include: other
        - include: otherHandlers
        - include: numbers

    keywords:
        - match: \bConstruct\b
          scope: keyword.control.nenyr

    scopes:
        - match: \b(Central|Layout|Module)\b
          scope: keyword.operator.nenyr

    declare:
        - match: \b(Declare|Extending|Deriving)\b
          scope: entity.name.function.nenyr

    handlers:
        - match: \b(Imports|Typefaces|Aliases|Variables|Breakpoints|Themes|Class|Animation)\b
          scope: variable.language.nenyr

    nested:
        - match: \b(Import|MobileFirst|DesktopFirst|Light|Dark|Important|PanoramicViewer|Stylesheet|Hover|Active|Focus|FirstChild|LastChild|FirstOfType|LastOfType|OnlyChild|OnlyOfType|TargetPseudoClass|Visited|Checked|Disabled|Enabled|ReadOnly|ReadWrite|PlaceholderShown|Valid|Invalid|Required|Optional|Fullscreen|FocusWithin|FirstLine|FirstLetter|Before|After|OutOfRange|Root|FirstPage|LeftPage|RightPage|Empty)\b
          scope: variable.nenyr

    properties:
        - match: \b(aspectRatio|accentColor|backdropFilter|content|gap|rowGap|scale|order|pointerEvents|margin|marginBottom|marginLeft|marginRight|marginTop|padding|paddingBottom|paddingLeft|paddingRight|paddingTop|height|width|filter|maxHeight|maxWidth|minHeight|minWidth|border|borderBottom|borderBottomColor|borderBottomStyle|borderBottomWidth|borderColor|borderLeft|borderLeftColor|borderLeftStyle|borderLeftWidth|borderRight|borderRightColor|borderRightStyles|borderRightWidth|borderStyle|borderTop|borderTopColor|borderTopStyle|borderTopWidth|borderWidth|outline|outlineColor|outlineStyle|outlineWidth|borderBottomLeftRadius|borderBottomRightRadius|borderImage|borderImageOutset|borderImageRepeat|borderImageSlice|borderImageSource|borderImageWidth|borderRadius|borderTopLeftRadius|borderTopRightRadius|boxDecorationBreak|boxShadow|background|backgroundAttachment|backgroundColor|backgroundImage|backgroundPosition|backgroundPositionX|backgroundPositionY|backgroundRepeat|backgroundClip|backgroundOrigin|backgroundSize|backgroundBlendMode|colorProfile|opacity|renderingIntent|font|fontFamily|fontSize|fontStyle|fontVariant|fontWeight|fontSizeAdjust|fontStretch|positioning|bottom|clear|clipPath|cursor|display|float|left|overflow|position|right|top|visibility|zIndex|color|direction|flexDirection|flexWrap|letterSpacing|lineHeight|lineBreak|textAlign|textDecoration|textIndent|textTransform|unicodeBidi|verticalAlign|whiteSpace|wordSpacing|textOutline|textOverflow|textShadow|textWrap|wordBreak|wordWrap|listStyle|listStyleImage|listStylePosition|listStyleType|borderCollapse|borderSpacing|captionSide|emptyCells|tableLayout|marqueeDirection|marqueePlayCount|marqueeSpeed|marqueeStyle|overflowX|overflowY|overflowStyle|rotation|boxAlign|boxDirection|boxFlex|boxFlexGroup|boxLines|boxOrdinalGroup|boxOrient|boxPack|alignmentAdjust|alignmentBaseline|baselineShift|dominantBaseline|dropInitialAfterAdjust|dropInitialAfterAlign|dropInitialBeforeAdjust|dropInitialBeforeAlign|dropInitialSize|dropInitialValue|inlineBoxAlign|lineStacking|lineStackingRuby|lineStackingShift|lineStackingStrategy|textHeight|columnCount|columnFill|columnGap|columnRule|columnRuleColor|columnRuleStyle|columnRuleWidth|columnSpan|columnWidth|columns|animation|animationName|animationDuration|animationTimingFunction|animationDelay|animationFillMode|animationIterationCount|animationDirection|animationPlayState|transform|transformOrigin|transformStyle|perspective|perspectiveOrigin|backfaceVisibility|transition|transitionProperty|transitionDuration|transitionTimingFunction|transitionDelay|orphans|pageBreakAfter|pageBreakBefore|pageBreakInside|widows|mark|markAfter|markBefore|phonemes|rest|restAfter|restBefore|voiceBalance|voiceDuration|voicePitch|voicePitchRange|voiceRate|voiceStress|voiceVolume|appearance|boxSizing|icon|navDown|navIndex|navLeft|navRight|navUp|outlineOffset|resize|quotes|rotate|translate|userSelect|writingMode|objectPosition|objectFit|justifySelf|justifyContent|justifyItems|alignSelf|alignContent|alignItems|grid|gridArea|gridAutoColumns|gridAutoFlow|gridAutoRows|gridColumn|gridColumnEnd|gridColumnStart|gridRow|gridRowEnd|gridRowStart|gridTemplate|gridTemplateAreas|gridTemplateColumns|gridTemplateRows|scrollbarColor|scrollbarWidth|scrollbarGutter)\b
          scope: keyword.operator.new.nenyr

    anime:
        - match: \b(Fraction|Progressive|From|Halfway|To)\b
          scope: constant.regexp.nenyr

    boolean:
        - match: \b(true|false)\b
          scope: constant.regexp.nenyr

    other:
        - match: \b([a-zA-Z_][a-zA-Z0-9_]*)\b\s*(?=:)
          scope: keyword.other.unit.nenyr

    otherHandlers:
        - match: \b[a-zA-Z_][a-zA-Z0-9_]*\b
          scope: entity.name.function.nenyr

    numbers:
        - match: \b\d+(\.\d+)?%?\b
          scope: entity.name.function.nenyr
"#.to_string()
}
