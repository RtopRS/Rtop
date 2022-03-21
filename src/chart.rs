pub struct Chart {
    cols: i32,
    rows: i32,
    show_pourcent: bool
}

impl Chart {
    pub fn display(&self, pourcents: &Vec<i32>) -> String {
        let mut data: Vec<i32>;
        if pourcents.len() >= ((self.cols * 2) - 1) as usize {
            data = pourcents.as_slice()[pourcents.len() + 2 - (self.cols * 2) as usize..].to_vec();
        } else {
            data = pourcents.to_vec();
        }
        data.reverse();

        let mut name_to_define: std::collections::HashMap<&str, &str> = std::collections::HashMap::new();

        let space_to_add = self.cols as f32 - (data.len() as f32 / 2.);

        name_to_define.insert("10", "⡀");
        name_to_define.insert("20", "⡄");
        name_to_define.insert("30", "⡆");
        name_to_define.insert("40", "⡇");

        name_to_define.insert("01", "⢀");
        name_to_define.insert("02", "⢠");
        name_to_define.insert("03", "⢰");
        name_to_define.insert("04", "⢸");

        name_to_define.insert("11", "⣀");
        name_to_define.insert("21", "⣄");
        name_to_define.insert("31", "⣆");
        name_to_define.insert("41", "⣇");

        name_to_define.insert("12", "⣠");
        name_to_define.insert("22", "⣤");
        name_to_define.insert("32", "⣦");
        name_to_define.insert("42", "⣧");

        name_to_define.insert("13", "⣰");
        name_to_define.insert("23", "⣴");
        name_to_define.insert("33", "⣶");
        name_to_define.insert("43", "⣷");

        name_to_define.insert("14", "⣸");
        name_to_define.insert("24", "⣼");
        name_to_define.insert("34", "⣾");
        name_to_define.insert("44", "⣿");

        name_to_define.insert("04", "⢀");
        name_to_define.insert("04", "⢠");
        name_to_define.insert("04", "⢰");
        name_to_define.insert("04", "⢸");
        name_to_define.insert("00", " ");

        let mut graph: String = "".to_string();

        let mut aa = self.rows;
        if self.show_pourcent {
            aa = self.rows - 1;
        }

        for row in 0..aa {
            let mut i = 0;
            while i < data.len() {
                let pourcent_one = data[i as usize];
                let mut tmp_one = pourcent_one as f32 / 100. * self.rows as f32 * 4.;
                if tmp_one < 1. {
                    tmp_one = 1.;
                }
                let mut full_block_to_add_one = tmp_one as i32 - (4 * row);

                let mut pourcent_two = 0;
                let mut tmp_two = pourcent_two as f32 / 100. * self.rows as f32 * 4.;
                let mut full_block_to_add_two = tmp_two as i32 - (4 * row);

                if i as usize + 1 < data.len() {    
                    pourcent_two = data[i as usize + 1];
                    tmp_two = pourcent_two as f32 / 100. * self.rows as f32 * 4.;
                    if tmp_two < 1. {
                        tmp_two = 1.;
                    }
                    full_block_to_add_two = tmp_two as i32 - (4 * row);
                }

                if full_block_to_add_one < 0 {
                    full_block_to_add_one = 0;
                }
                if full_block_to_add_two < 0 {
                    full_block_to_add_two = 0;
                }

                if full_block_to_add_one > 4 {
                    full_block_to_add_one = 4;
                }
                if full_block_to_add_two > 4 {
                    full_block_to_add_two = 4;
                }


                graph = format!("{}{}", graph, name_to_define[&format!("{}{}",full_block_to_add_two, full_block_to_add_one).to_string()[..]]);
                i += 2
            }


            graph = format!("\n{}{}", graph, " ".repeat(space_to_add as usize));
        }

        if self.show_pourcent && data.len() != 0 {
            graph = format!("{}%{}{}", graph, data[0].to_string().chars().rev().collect::<String>()," ".repeat((self.cols - data[0].to_string().chars().count() as i32 - 2) as usize));
        }

        graph.chars().rev().collect::<String>()
    }

    pub fn new(cols: i32, rows: i32, show_pourcent: bool) -> Chart {
        Chart{cols: cols, rows: rows, show_pourcent: show_pourcent}
    }

    pub fn resize(&mut self, cols: i32, rows: i32) {
        self.cols = cols;
        self.rows = rows;
    }
}
