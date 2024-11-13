#[derive(Clone, PartialEq, Debug)]
pub struct ShellscapeArea {
    left: u16,
    right: u16,
    top: u16,
    bottom: u16,
}

impl ShellscapeArea {
    pub fn new(left: u16, right: u16, top: u16, bottom: u16) -> Self {
        Self {
            left,
            right,
            top,
            bottom,
        }
    }

    pub fn left(&self) -> u16 {
        self.left
    }

    pub fn right(&self) -> u16 {
        self.right
    }

    pub fn top(&self) -> u16 {
        self.top
    }

    pub fn bottom(&self) -> u16 {
        self.bottom
    }
}
