use ncurses::*;
use std::fmt::Write;
use unicode_segmentation::UnicodeSegmentation;

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
        let win_inner = derwin(win_box, height - 2, width - 4, 1, 2);
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

        let mut trimmed_text = String::new();
        let mut formated_string = String::new();

        for part in content.split("[[EFFECT_").collect::<Vec<&str>>() {
            if part.contains("]]") {
                let effect: Vec<&str> = part.split("]]").collect();

                let aa: String = part.chars().skip(effect[0].len() + 2).collect();
                trimmed_text += &aa;
            } else {
                trimmed_text += part;
            }
        }

        for (i, line) in trimmed_text.split('\n').into_iter().enumerate() {
            if line.graphemes(true).count() == self.width as usize - 4 && line.ends_with('\n') {

                let tmp = content.split('\n').collect::<Vec<&str>>()[i].split("");

                formated_string += &tmp.take(line.graphemes(true).count() - 1).into_iter().collect::<String>();
            } else if line.graphemes(true).count() == self.width as usize - 4 {
                formated_string += content.split('\n').collect::<Vec<&str>>()[i];
            } else {
                writeln!(&mut formated_string, "{}", content.split('\n').collect::<Vec<&str>>()[i]).unwrap();
            }
        }



        for el in formated_string.split("[[EFFECT_").collect::<Vec<&str>>() {
            if el.contains("]]") {
                let effect: Vec<&str> = el.split("]]").collect();

                let attr = self.get_attr_from_string(effect[0]);
                if let Some(attr) = attr {
                    if effect[0].starts_with("COLOR") {
                        if !color_applied.is_empty() && color_applied[color_applied.len() - 1] == attr {
                            color_applied.pop();
                        } else {
                            color_applied.push(attr);
                        }
                    } else if !effect_applied.is_empty() {
                        effect_applied.pop();
                    } else {
                        effect_applied.push(attr);
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
                let aa: String = el.chars().skip(effect[0].len() + 2).collect();
                waddstr(self.inner_window, &aa);
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
                if let Some(attr) = attr {
                    if effect[0].starts_with("COLOR") {
                        if !color_applied.is_empty() && color_applied[color_applied.len() - 1] == attr {
                            color_applied.pop();
                        } else {
                            color_applied.push(attr);
                        }
                    } else if !effect_applied.is_empty() {
                        effect_applied.pop();
                    } else {
                        effect_applied.push(attr);
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
        let mut tmp = attribute.chars().collect::<Vec<char>>();
        tmp.retain(|&c| c == '_');

        if attribute.starts_with("COLOR_") && tmp.len() == 2 {
            let foreground = match attribute.split('_').collect::<Vec<&str>>()[1] {
                "RED" => COLOR_RED,
                "GREEN" => COLOR_GREEN,
                "YELLOW" => COLOR_YELLOW,
                "BLUE" => COLOR_BLUE,
                "MAGENTA" => COLOR_MAGENTA,
                "CYAN" => COLOR_CYAN,
                "WHITE" => COLOR_WHITE,
                "BLACK" => COLOR_BLACK,
                _ => -1
            };
            let background = match attribute.split('_').collect::<Vec<&str>>()[2] {
                "RED" => COLOR_RED,
                "GREEN" => COLOR_GREEN,
                "YELLOW" => COLOR_YELLOW,
                "BLUE" => COLOR_BLUE,
                "MAGENTA" => COLOR_MAGENTA,
                "CYAN" => COLOR_CYAN,
                "WHITE" => COLOR_WHITE,
                "BLACK" => COLOR_BLACK,
                _ => -1
            };
            
            init_pair(foreground * 10 + background, foreground, background);
            std::option::Option::from(COLOR_PAIR(foreground * 10 + background))
        } else {
            match attribute {
                "REVERSE" => std::option::Option::from(ncurses::A_REVERSE()),
                "BOLD" => std::option::Option::from(ncurses::A_BOLD()),
                "ITALIC" => std::option::Option::from(ncurses::A_ITALIC()),
                "DIMMED" => std::option::Option::from(ncurses::A_DIM()),
                "UNDERLINE" => std::option::Option::from(ncurses::A_UNDERLINE()),
                "COLOR_RED" => std::option::Option::from(COLOR_PAIR(1)),
                "COLOR_GREEN" => std::option::Option::from(COLOR_PAIR(2)),
                "COLOR_YELLOW" => std::option::Option::from(COLOR_PAIR(3)),
                "COLOR_BLUE" => std::option::Option::from(COLOR_PAIR(4)),
                "COLOR_MAGENTA" => std::option::Option::from(COLOR_PAIR(5)),
                "COLOR_CYAN" => std::option::Option::from(COLOR_PAIR(6)),
                "COLOR_WHITE" => std::option::Option::from(COLOR_PAIR(7)),
                "COLOR_BLACK" => std::option::Option::from(COLOR_PAIR(8)),
                _ => std::option::Option::None 
            }
        }
    }
}