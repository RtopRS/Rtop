use pancurses::*;

pub struct Window {
    pub height: i32,
    pub width: i32,
    pub x: i32,
    pub y: i32,
    curse_window: pancurses::Window,
    inner_window: pancurses::Window,
    text_color: ColorPair,
    pub title: String,
    border_color: ColorPair
}

impl Window {
    pub fn new(height: i32, width: i32, x: i32, y: i32, border_color: ColorPair, text_color: ColorPair, title: String) -> Window {
        let win_box = newwin(height, width, y, x);
        let win_inner = win_box.derwin(height - 2, width - 2, 1, 1).unwrap();
        let new_win = Window{height: height, width: width, x: x, y: y, curse_window: win_box, inner_window: win_inner, text_color: text_color, title: title, border_color: border_color};
        new_win.curse_window.attrset(border_color);
        new_win.inner_window.attron(text_color);
        new_win.draw_border();
        new_win.curse_window.refresh();
        return new_win;
    }

    pub fn refresh(&self) {
        self.curse_window.refresh();
        self.inner_window.refresh();
    }

    pub fn write(&self, content: &str) {
        self.draw_border();
        self.inner_window.erase();
        let mut aa = false;
        for line in content.split("[[REVERSE]]") {
            self.inner_window.addstr(line);
            if aa {
                self.inner_window.attroff(pancurses::A_REVERSE);
            } else {
                self.inner_window.attron(pancurses::A_REVERSE);
            }
            aa = !aa;
        }
        self.inner_window.attroff(pancurses::A_REVERSE);
    }

    fn draw_border(&self) {
        self.curse_window.mvaddstr(0, 1, "─".to_string().repeat((self.width - 2) as usize));
        self.curse_window.mvaddstr(self.height - 1, 1, "─".to_string().repeat((self.width - 2) as usize));
        for i in 1..self.height - 1 {
            self.curse_window.mvaddstr(i, 0, "│");
            self.curse_window.mvaddstr(i, self.width - 1, "│");
        }
        self.curse_window.mvaddstr(0, 0, "┌");
        self.curse_window.mvaddstr(self.height - 1, self.width - 1, "┘");
        self.curse_window.mvaddstr(self.height - 1, 0, "└");
        self.curse_window.mvaddstr(0, self.width - 1, "┐");
        self.curse_window.attroff(self.border_color);
        self.curse_window.mvaddstr(0, 2, format!(" {} ", &self.title));
        self.curse_window.attrset(self.border_color);
    }

    pub fn resize(&mut self, height: i32, width: i32) {
        self.curse_window.resize(height, width);
        self.height = height;
        self.width = width;
    }

    pub fn deplace(&mut self, x: i32, y: i32) {
        self.curse_window.mvwin(y, x);
        self.inner_window = self.curse_window.derwin(self.height - 2,self.width - 2, 1, 1).unwrap();
        self.inner_window.attron(self.text_color);
    }
}