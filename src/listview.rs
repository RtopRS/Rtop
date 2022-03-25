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
    pub fn new(height: i32, width: i32, items: &Vec<ListItem>, primary_key: &str, secondary_keys: Vec<String>) -> ListView {
        ListView{height: height, width: width, counter: 0, items: items.to_vec(), primary_key: primary_key.to_string(), secondary_keys: secondary_keys, selected_line: 1, start_index: 0}
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
            if self.selected_line != self.height - 1 {
                self.selected_line += 1;
            } else {
                self.start_index += 1;
            }
        }
    }

    pub fn to_last(&mut self) {
        self.counter = self.items.len() as i32 - 1;
        self.selected_line = self.counter + 1;
        if self.selected_line > self.height - 1 {
            self.start_index = self.counter - (self.height - 2);
            self.selected_line = self.height - 1;
        }
    }

    pub fn to_first(&mut self) {
        self.counter = 0;
        self.selected_line = 1;
        self.start_index = 0;
    }

    pub fn display(&mut self) -> String {
        self.items.sort_by(|a, b| (b.data["Count"].parse::<usize>().unwrap()).cmp(&(a.data["Count"].parse::<usize>().unwrap())));
        let mut secondary_cols = "".to_string();
        for key in &self.secondary_keys {
            secondary_cols = format!("{}{}   ", secondary_cols, key)
        }
        let mut aa = format!("{}{}{}\n", self.primary_key, " ".repeat(self.width as usize - self.primary_key.len() - secondary_cols.len() - 1), secondary_cols);
        let mut displayed_items = &self.items[..];
        if displayed_items.len() > (self.height - 1) as usize {
            displayed_items = &self.items[self.start_index as usize..(self.start_index + self.height - 1) as usize]
        }
        let mut i = 1;
        for item in displayed_items {
            if i == self.selected_line {
                aa = format!("{}[[REVERSE]]{}{}[[REVERSE]]", aa, item.name, " ".repeat(self.width as usize - item.name.len() - secondary_cols.len() - 1))
            } else {
                aa = format!("{}{}{}", aa, item.name, " ".repeat(self.width as usize - item.name.len() - secondary_cols.len() - 1))
            }
            for col in &self.secondary_keys {
                if item.data.contains_key(col) {
                    aa = format!("{}{}{}", aa, item.data[col], " ".repeat((col.len() + 3) - item.data[col].len()));
                } else {
                    aa = format!("{}{}", aa, " ".repeat(col.len() + 3));
                }
                
            }
            aa = format!("{}\n", aa);

            i += 1;
        }   
        aa
    }

    pub fn resize(&mut self, height: i32, width: i32) {
        self.height = height;
        self.width = width;
        if self.selected_line > self.height - 1 {
            self.selected_line = self.height - 1;
            self.start_index = self.counter - (self.height - 2);
        }
    }

    pub fn update_items(&mut self, items: &Vec<ListItem>) {
        self.items = items.to_vec();
        if items.len() < self.counter as usize {
            self.counter = items.len() as i32 - 1;
        }
    }
}

#[derive(Clone)]
pub struct ListItem {
    pub name: String,
    pub data: std::collections::HashMap<String, String>
}

impl ListItem {
    pub fn new(name: &str, data: &std::collections::HashMap<String, String>) -> ListItem {
        let new_item = ListItem{name: name.to_string(), data: data.clone()};
        new_item
    }
}