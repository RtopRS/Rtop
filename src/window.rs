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

        let mut color_applied = vec!();
        let mut effect_applied = vec!();

        for el in content.split("[[EFFECT_").collect::<Vec<&str>>() {
            if el.contains("]]") {
                let effect: Vec<&str> = el.split("]]").collect();

                let attr = self.get_attr_from_string(effect[0]);
                if attr.is_some() {
                    if effect[0].starts_with("COLOR") {
                        if !color_applied.is_empty() && color_applied[color_applied.len() - 1] == attr.unwrap() {
                            color_applied.pop();
                        } else {
                            color_applied.push(attr.unwrap());
                        }
                    } else {
                        if !effect_applied.is_empty() && effect_applied[effect_applied.len() - 1] == attr.unwrap() {
                            effect_applied.pop();
                        } else {
                            if !effect_applied.is_empty() {
                                effect_applied.pop();
                            } else {
                                effect_applied.push(attr.unwrap());
                            }
                        }
                    }
                }


                if effect_applied.is_empty() {
                    wattr_off(self.inner_window, A_ATTRIBUTES());
                } else {
                    wattr_on(self.inner_window, effect_applied[effect_applied.len() - 1]);
                }
                if color_applied.is_empty() {
                    wattr_on(self.inner_window, self.text_color);
                } else {
                    wattr_on(self.inner_window, color_applied[color_applied.len() - 1]);
                }

                let a: String = el.chars().skip(effect[0].len() + 2).collect();
                waddstr(self.inner_window, &a);
            } else {
                waddstr(self.inner_window, el);
            }

        }

        wattrset(self.inner_window, self.text_color);
    }

    fn draw_border(&self) {
        wattron(self.curse_window, self.border_color);
        box_(self.curse_window, 0, 0);
        wattroff(self.curse_window, self.border_color);
        mvwaddstr(self.curse_window, 0, 2, " ");

        let mut color_applied = vec!();
        let mut effect_applied = vec!();
        for el in self.title.split("[[EFFECT_").collect::<Vec<&str>>() {
            if el.contains("]]") {
                let effect: Vec<&str> = el.split("]]").collect();

                let attr = self.get_attr_from_string(effect[0]);
                if attr.is_some() {
                    if effect[0].starts_with("COLOR") {
                        if !color_applied.is_empty() && color_applied[color_applied.len() - 1] == attr.unwrap() {
                            color_applied.pop();
                        } else {
                            color_applied.push(attr.unwrap());
                        }
                    } else {
                        if !effect_applied.is_empty() && effect_applied[effect_applied.len() - 1] == attr.unwrap() {
                            effect_applied.pop();
                        } else if !effect_applied.is_empty() {
                                effect_applied.pop();
                        } else {
                            effect_applied.push(attr.unwrap());
                        }
                    }
                }

                if effect_applied.is_empty() {
                    wattr_off(self.curse_window, A_ATTRIBUTES());
                } else {
                    wattr_on(self.curse_window, effect_applied[effect_applied.len() - 1]);
                }
                if color_applied.is_empty() {
                    wattr_off(self.curse_window, self.border_color);
                } else {
                    wattr_on(self.curse_window, color_applied[color_applied.len() - 1]);
                }

                let a: String = el.chars().skip(effect[0].len() + 2).collect();
                waddstr(self.curse_window, &a);
            } else {
                waddstr(self.curse_window, el);
            }

        }

        waddstr(self.curse_window, " ");
        wattrset(self.curse_window, ncurses::A_NORMAL());
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
        self.title = title;
    }

    pub fn set_border_color(&mut self, border_color: attr_t) {
        self.border_color = border_color;
        self.draw_border();
    }

    fn get_attr_from_string(&self, attribute: &str) -> std::option::Option<attr_t> {
        if attribute == "REVERSE" {
            std::option::Option::from(ncurses::A_REVERSE())
        } else if attribute == "BOLD" {
            std::option::Option::from(ncurses::A_BOLD())
        } else if attribute == "COLOR_GREEN" {
            std::option::Option::from(COLOR_PAIR(1))
        } else if attribute == "COLOR_RED" {
            std::option::Option::from(COLOR_PAIR(3))
        } else if attribute == "COLOR_GREEN_GREY" {
            std::option::Option::from(COLOR_PAIR(4))
        } else if attribute == "COLOR_BLUE" {
            std::option::Option::from(COLOR_PAIR(2))
        } else {
            std::option::Option::None
        }
    }
}