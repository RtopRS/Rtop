use ncurses::*;

pub struct Window {
    pub height: i32,
    pub width: i32,
    pub x: i32,
    pub y: i32,
    curse_window: ncurses::WINDOW,
    inner_window: ncurses::WINDOW,
    text_color: attr_t,
    border_color: attr_t,
    title: String
}

impl Window {
    pub fn new(height: i32, width: i32, x: i32, y: i32, border_color: attr_t, text_color: attr_t, title: String) -> Window {
        let win_box = newwin(height, width, y, x);
        let win_inner = derwin(win_box, height - 2, width - 2, 1, 1);
        let new_win = Window{height, width, x, y, curse_window: win_box, inner_window: win_inner, text_color, border_color, title};
        wattrset(new_win.inner_window, text_color);
        new_win.draw_border();
        new_win
    }

    pub fn refresh(&self) {
        wrefresh(self.curse_window);
        wrefresh(self.inner_window);
    }

    pub fn write(&self, content: &str) {
        werase(self.inner_window);
        let mut aa = false;
        for line in content.split("[[REVERSE]]") {
            waddstr(self.inner_window, line);
            if aa {
                wattroff(self.inner_window, ncurses::A_REVERSE());
            } else {
                wattron(self.inner_window, ncurses::A_REVERSE());
            }
            aa = !aa;
        }
        wattroff(self.inner_window, ncurses::A_REVERSE());
    }

    fn draw_border(&self) {
        wattron(self.curse_window, self.border_color);
        box_(self.curse_window, 0, 0);
        wattroff(self.curse_window, self.border_color);
        mvwaddstr(self.curse_window, 0, 2, &format!(" {} ", self.title));
        wattron(self.curse_window, self.border_color);
    }

    pub fn resize(&mut self, height: i32, width: i32) {
        self.height = height;
        self.width = width;
    }

    pub fn deplace(&mut self, x: i32, y: i32) {
        delwin(self.curse_window);
        delwin(self.inner_window);
        self.curse_window = newwin(self.height, self.width, y, x);
        self.inner_window = derwin(self.curse_window, self.height - 2, self.width - 2, 1, 1);
        wattron(self.inner_window, self.text_color);
        self.draw_border();
    }

    pub fn set_title(&mut self, title: String) {
        wattroff(self.curse_window, self.border_color);
        mvwaddstr(self.curse_window, 0, 2, &format!(" {} ", title));
        wattron(self.curse_window, self.border_color);
        self.title = title;
    }
}