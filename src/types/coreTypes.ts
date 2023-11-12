export interface ExtractGaladrielClassesType {
    (classes: Record<string, Record<string, any>>): string[];
}

export interface CoreStaticStylesType {
    [key: string]: (options: {
        extractGaladrielClasses: ExtractGaladrielClassesType;
    }) => any;
}

export interface ClassesObjectType {
    [key: string]: any;
}

export interface GaladrielParamsType {
    [key: string]: string[];
}

export interface ExtractGaladrielCSSClassesType {
    (classes: Record<string, Record<string, any>>): any;
}

export interface ButtonColorProps {
    bgColor: string;
    borderColor: string;
    bsColorOne: string;
    bsColorTwo: string;
    filterColor: string;
    textColor: string;
}

export interface NavbarColorProps {
    bgColorStart: string;
    bgColorEnd: string;
    textColor: string;
    hoverColor: string;
    shadowColor: string;
}

export interface FooterColorProps {
    bgColorStart: string;
    bgColorEnd: string;
    textColorOne: string;
    textColorTwo: string;
    shadowColor: string;
}

export interface SimplifiedButtonProps {
    bgColorStart: string;
    bgColorEnd: string;
    borderColor: string;
    focusColor: string;
    shadowColor: string;
}

export type HTMLElementTagName =
    | "a"
    | "abbr"
    | "address"
    | "area"
    | "article"
    | "aside"
    | "audio"
    | "b"
    | "base"
    | "bdi"
    | "bdo"
    | "blockquote"
    | "body"
    | "br"
    | "button"
    | "canvas"
    | "caption"
    | "cite"
    | "code"
    | "col"
    | "colgroup"
    | "data"
    | "datalist"
    | "dd"
    | "del"
    | "details"
    | "dfn"
    | "dialog"
    | "div"
    | "dl"
    | "dt"
    | "em"
    | "embed"
    | "fieldset"
    | "figcaption"
    | "figure"
    | "footer"
    | "form"
    | "h1"
    | "h2"
    | "h3"
    | "h4"
    | "h5"
    | "h6"
    | "head"
    | "header"
    | "hgroup"
    | "hr"
    | "html"
    | "i"
    | "iframe"
    | "img"
    | "input"
    | "ins"
    | "kbd"
    | "label"
    | "legend"
    | "li"
    | "link"
    | "main"
    | "map"
    | "mark"
    | "meta"
    | "meter"
    | "nav"
    | "noscript"
    | "object"
    | "ol"
    | "optgroup"
    | "option"
    | "output"
    | "p"
    | "param"
    | "picture"
    | "pre"
    | "progress"
    | "q"
    | "rb"
    | "rp"
    | "rt"
    | "rtc"
    | "ruby"
    | "s"
    | "samp"
    | "script"
    | "section"
    | "select"
    | "slot"
    | "small"
    | "source"
    | "span"
    | "strong"
    | "style"
    | "sub"
    | "summary"
    | "sup"
    | "table"
    | "tbody"
    | "td"
    | "template"
    | "textarea"
    | "tfoot"
    | "th"
    | "thead"
    | "time"
    | "title"
    | "tr"
    | "track"
    | "u"
    | "ul"
    | "var"
    | "video"
    | "wbr";
