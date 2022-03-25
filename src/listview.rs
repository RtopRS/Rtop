pub struct ListView {
    height: i32,
    width: i32,
    items: Vec<ListItem>,
    primary_key: String,
    secondary_keys: Vec<String>,
    selected_line : i32,
    start_index: i32,

    counter: i32
}

impl ListView {
    pub fn new(height: i32, width: i32, items: Vec<ListItem>, primary_key: &str, secondary_keys: Vec<String>) -> ListView {
        ListView{height: height, width: width, counter: 0, items: items, primary_key: primary_key.to_string(), secondary_keys: secondary_keys, selected_line: 1, start_index: 0}
    }

    pub fn previous(&mut self) {
        if self.counter > 0 {
            self.counter -= 1;
            if self.selected_line > 1 {
                self.selected_line -= 1;
            } else {
                self.start_index -= 1;
            }
        }
    }

    pub fn next(&mut self) {
        if self.counter < self.items.len() as i32 - 1{
            self.counter += 1;
            if self.selected_line != self.height - 2 {
                self.selected_line += 1;
            } else {
                self.start_index += 1;
            }
        }
    }

    pub fn to_last(&mut self) {
        self.counter = self.items.len() as i32 - 1;
        self.selected_line = self.counter + 1;
        if self.selected_line > self.height - 2 {
            self.start_index = self.counter - (self.height - 3);
            self.selected_line = self.height - 2;
        }
    }

    pub fn to_first(&mut self) {
        self.counter = 0;
        self.selected_line = 1;
        self.start_index = 0;
    }

    pub fn display(&self) -> String {
        let mut aa = format!("{}                         ", self.primary_key);
        for key in &self.secondary_keys {
            aa = format!("{}{}    ", aa, key)
        }
        aa = format!("{}\ncurrent_items_index: {}, selected_line: {}, start_index: {}, lines: {}\n", aa, self.counter, self.selected_line, self.start_index, self.height - 2);
        let mut displayed_items = &self.items[..];
        if displayed_items.len() > (self.height - 2) as usize {
            displayed_items = &self.items[self.start_index as usize..(self.start_index + self.height - 2) as usize]
        }
        let mut i = 1;
        for item in displayed_items {
            if i == self.selected_line {
                aa = format!("{}[[REVERSE]]{}{}[[REVERSE]]\n", aa, item.name, " ".repeat(self.width as usize - item.name.len() - 1))
            } else {
                aa = format!("{}{}\n", aa, item.name)
            }
            i += 1;
        }   
        aa
    }

    pub fn resize(&mut self, height: i32, width: i32) {
        self.height = height;
        self.width = width;
        if self.selected_line > self.height - 2 {
            self.selected_line = self.height - 2;
            self.start_index = self.counter - (self.height - 3);
        }
    }
}

pub struct ListItem {
    pub name: String,
}